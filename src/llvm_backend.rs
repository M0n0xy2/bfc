use llvm;
use llvm::prelude::*;
use llvm::execution_engine::LLVMExecutionEngineRef;
use std;
use std::os::raw::c_char;
use std::ffi::CString;

use ir::Atom;

pub fn jit_ir(ir: &Vec<Atom>, opt: bool) -> Result<(), CString> {
    unsafe {
        let mut builder = LLVMBackendBuilder::new();
        builder.generate(ir);
        let mut backend = builder.build()?;
        if opt {
            backend.do_opt();
        }
        backend.run_jit();
    }
    Ok(())
}

const MEM_SIZE: isize = 30000;

#[derive(Debug, Clone)]
struct LLVMBackendBuilder {
    module: LLVMModuleRef,
    brainfuck_fn: LLVMValueRef,
    builder: LLVMBuilderRef,
    memory: LLVMValueRef,
    ptr: LLVMValueRef,
    putchar_fn: LLVMValueRef,
    getchar_fn: LLVMValueRef,
    free_fn: LLVMValueRef,
}

impl LLVMBackendBuilder {
    unsafe fn new() -> LLVMBackendBuilder {
        macro_rules! add_function {
            ($module:expr, $name:expr, $ret_ty:expr, $args:expr) => {
                {
                    let mut args = $args;
                    let args_len = args.len();
                    let fn_ty = llvm::core::LLVMFunctionType(
                        $ret_ty,
                        args.as_mut_ptr(),
                        args_len as _,
                        0
                    );
                    llvm::core::LLVMAddFunction(
                        $module,
                        $name.as_ptr() as *const _,
                        fn_ty
                    )
                }
            };
            ($module:expr, $name:expr, $ret_ty:expr, []) => {
                {
                    let fn_ty = llvm::core::LLVMFunctionType(
                        $ret_ty,
                        std::ptr::null_mut(),
                        0,
                        0
                    );
                    llvm::core::LLVMAddFunction(
                        $module,
                        $name.as_ptr() as *const _,
                        fn_ty
                    )
                }
            }
        }


        let module = llvm::core::LLVMModuleCreateWithName(
            b"main_module\0".as_ptr() as *const _
        );

        let builder = llvm::core::LLVMCreateBuilder();
        
        let i8_ty = llvm::core::LLVMInt8Type();
        let i32_ty = llvm::core::LLVMInt32Type();
        let void_ty = llvm::core::LLVMVoidType();
        let i8_ptr_ty = llvm::core::LLVMPointerType(i8_ty, 0);

        let putchar_fn = add_function!(module, b"putchar\0", i32_ty, [i32_ty]);
        let getchar_fn = add_function!(module, b"getchar\0", i32_ty, []);
        let calloc_fn = add_function!(
            module,
            b"calloc\0",
            i8_ptr_ty,
            [i32_ty, i32_ty]
        );
        let free_fn = add_function!(module, b"free\0", void_ty, [i8_ptr_ty]);

        let bf_fn_ty = llvm::core::LLVMFunctionType(
            void_ty,
            std::ptr::null_mut(),
            0,
            0
        );
        let bf_fn = llvm::core::LLVMAddFunction(
            module,
            b"brainfuck\0".as_ptr() as *const _,
            bf_fn_ty
        );

        let entry_bb = llvm::core::LLVMAppendBasicBlock(
            bf_fn,
            b"entry\0".as_ptr() as *const _
        );
        llvm::core::LLVMPositionBuilderAtEnd(builder, entry_bb);

        
        let memory = llvm::core::LLVMBuildCall(
            builder,
            calloc_fn,
            [utils::get_int32_const(MEM_SIZE), utils::get_int32_const(1)].as_mut_ptr(),
            2,
            b"memory\0".as_ptr() as *const _
        );

        let ptr = llvm::core::LLVMBuildAlloca(
            builder,
            llvm::core::LLVMPointerType(i8_ty, 0),
            b"ptr_cell\0".as_ptr() as *const _
        );
        let ptr_init_value = llvm::core::LLVMBuildGEP(
            builder,
            memory,
            [utils::get_int32_const(0)].as_mut_ptr(),
            1,
            b"ptr_init_value\0".as_ptr() as *const _
        );

        llvm::core::LLVMBuildStore(
            builder,
            ptr_init_value,
            ptr
        );

        LLVMBackendBuilder {
            module,
            brainfuck_fn: bf_fn,
            builder,
            memory,
            ptr,
            putchar_fn,
            getchar_fn,
            free_fn,
        }
    }

    unsafe fn generate(&mut self, ir: &Vec<Atom>) {
        macro_rules! offset_ptr {
            ($builder:expr, $ptr:expr, $offset:expr) => {
                {
                    let ptr_value = llvm::core::LLVMBuildLoad(
                        $builder,
                        $ptr,
                        b"ptr\0".as_ptr() as *const _
                    );
                    llvm::core::LLVMBuildGEP(
                        $builder,
                        ptr_value,
                        [utils::get_int32_const($offset)].as_mut_ptr(),
                        1,
                        b"ptr\0".as_ptr() as *const _
                    )
                }
            }
        }

        for atom in ir {
            match *atom {
                Atom::MovePtr(offset) => {
                    let new_ptr_value = offset_ptr!(self.builder, self.ptr, offset);
                    llvm::core::LLVMBuildStore(
                        self.builder,
                        new_ptr_value,
                        self.ptr
                    );
                },
                Atom::SetValue(value, offset) => {
                    let real_ptr = offset_ptr!(self.builder, self.ptr, offset);
                    llvm::core::LLVMBuildStore(
                        self.builder,
                        utils::get_int8_const(value),
                        real_ptr
                    );
                },
                Atom::IncValue(inc, offset) => {
                    let real_ptr = offset_ptr!(self.builder, self.ptr, offset);

                    let value = llvm::core::LLVMBuildLoad(
                        self.builder,
                        real_ptr,
                        b"value\0".as_ptr() as *const _
                    );

                    let value = llvm::core::LLVMBuildAdd(
                        self.builder,
                        value,
                        utils::get_int8_const(inc),
                        b"value\0".as_ptr() as *const _
                    );

                    llvm::core::LLVMBuildStore(self.builder, value, real_ptr);
                },
                Atom::Print(offset) => {
                    let real_ptr = offset_ptr!(self.builder, self.ptr, offset);
                    let value = llvm::core::LLVMBuildLoad(
                        self.builder,
                        real_ptr,
                        b"value\0".as_ptr() as *const _
                    );

                    let value = llvm::core::LLVMBuildZExt(
                        self.builder,
                        value,
                        llvm::core::LLVMInt32Type(),
                        b"value\0".as_ptr() as *const _
                    );

                    llvm::core::LLVMBuildCall(
                        self.builder,
                        self.putchar_fn,
                        [value].as_mut_ptr(),
                        1,
                        b"value\0".as_ptr() as *const _
                    );
                },
                Atom::Read(offset) => {
                    let real_ptr = offset_ptr!(self.builder, self.ptr, offset);
                    let c = llvm::core::LLVMBuildCall(
                        self.builder,
                        self.getchar_fn,
                        std::ptr::null_mut(),
                        0,
                        b"value\0".as_ptr() as *const _
                    );
                    let value = llvm::core::LLVMBuildTrunc(
                        self.builder,
                        c,
                        llvm::core::LLVMInt8Type(),
                        b"value\0".as_ptr() as *const _
                    );
                    llvm::core::LLVMBuildStore(
                        self.builder,
                        value,
                        real_ptr
                    );
                },
                Atom::Multiply(factor, offset) => {
                    let base_ptr = offset_ptr!(self.builder, self.ptr, 0);
                    let offset_ptr = offset_ptr!(self.builder, self.ptr, offset);

                    let base_value = llvm::core::LLVMBuildLoad(
                        self.builder,
                        base_ptr,
                        b"offset_value\0".as_ptr() as *const _
                    );
                    let base_value = llvm::core::LLVMBuildMul(
                        self.builder,
                        base_value,
                        utils::get_int8_const(factor),
                        b"factored_value\0".as_ptr() as *const _
                    );
                    let offset_value = llvm::core::LLVMBuildLoad(
                        self.builder,
                        offset_ptr,
                        b"base_value\0".as_ptr() as *const _
                    );
                    let value = llvm::core::LLVMBuildAdd(
                        self.builder,
                        base_value,
                        offset_value,
                        b"value\0".as_ptr() as *const _
                    );
                    llvm::core::LLVMBuildStore(
                        self.builder,
                        value,
                        offset_ptr
                    );
                },
                Atom::Loop(ref sub) => {
                    let loop_bb = llvm::core::LLVMAppendBasicBlock(
                        self.brainfuck_fn,
                        b"loop\0".as_ptr() as *const _
                    );
                    llvm::core::LLVMBuildBr(self.builder, loop_bb);
                    let then_bb = llvm::core::LLVMAppendBasicBlock(
                        self.brainfuck_fn,
                        b"then\0".as_ptr() as *const _
                    );
                    let exit_bb = llvm::core::LLVMAppendBasicBlock(
                        self.brainfuck_fn,
                        b"exit\0".as_ptr() as *const _
                    );

                    llvm::core::LLVMPositionBuilderAtEnd(self.builder, loop_bb);

                    let ptr = offset_ptr!(self.builder, self.ptr, 0);
                    let value = llvm::core::LLVMBuildLoad(
                        self.builder,
                        ptr,
                        b"value\0".as_ptr() as *const _
                    );
                    let cond = llvm::core::LLVMBuildIsNotNull(
                        self.builder,
                        value,
                        b"cond\0".as_ptr() as *const _
                    );
                    llvm::core::LLVMBuildCondBr(
                        self.builder,
                        cond,
                        then_bb,
                        exit_bb
                    );

                    llvm::core::LLVMPositionBuilderAtEnd(self.builder, then_bb);
                    self.generate(sub);
                    llvm::core::LLVMBuildBr(self.builder, loop_bb);

                    let last_bb = llvm::core::LLVMGetLastBasicBlock(self.brainfuck_fn);
                    llvm::core::LLVMMoveBasicBlockAfter(exit_bb, last_bb);
                    llvm::core::LLVMPositionBuilderAtEnd(self.builder, exit_bb);
                }
            }
        }
    }

    

    fn build(self) -> Result<LLVMBackend, CString> {
        unsafe {
            llvm::core::LLVMBuildCall(
                self.builder,
                self.free_fn,
                [self.memory].as_mut_ptr(),
                1,
                b"\0".as_ptr() as *const _
            );
            llvm::core::LLVMBuildRetVoid(self.builder);

            let mut error: *mut c_char = std::ptr::null_mut();
            if llvm::analysis::LLVMVerifyModule(
                self.module,
                llvm::analysis::LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &mut error
                ) != 0 {
                return Err(CString::from_raw(error))
            }
            
            LLVMBackend::new(self.module, self.brainfuck_fn)
        }
    }
}

impl Drop for LLVMBackendBuilder {
    fn drop(&mut self) {
        unsafe {
            llvm::core::LLVMDisposeBuilder(self.builder);
        }
    }
}

#[derive(Debug, Clone)]
struct LLVMBackend {
    exec_engine: LLVMExecutionEngineRef,
    module: LLVMModuleRef,
    brainfuck_fn: LLVMValueRef,
}

impl LLVMBackend {
    fn new(module: LLVMModuleRef, brainfuck_fn: LLVMValueRef)
        -> Result<LLVMBackend, CString> {
        use llvm::execution_engine::LLVMMCJITCompilerOptions;

        let mut exec_engine: LLVMExecutionEngineRef = std::ptr::null_mut();
        unsafe {
            let mut error: *mut c_char = std::ptr::null_mut();
            let mut options: LLVMMCJITCompilerOptions = std::mem::zeroed();
            let options_size = 
                std::mem::size_of::<LLVMMCJITCompilerOptions>();
            llvm::execution_engine::LLVMInitializeMCJITCompilerOptions(
                &mut options,
                options_size
            );

            options.OptLevel = 3;

            llvm::target::LLVM_InitializeNativeTarget();
            llvm::target::LLVM_InitializeNativeAsmPrinter();
            llvm::target::LLVM_InitializeNativeAsmParser();
            llvm::execution_engine::LLVMLinkInMCJIT();
            if llvm::execution_engine::LLVMCreateMCJITCompilerForModule(
                &mut exec_engine,
                module,
                &mut options,
                options_size,
                &mut error
                ) != 0 {
                return Err(CString::from_raw(error));
            }
        }

        Ok(LLVMBackend {
            exec_engine,
            module,
            brainfuck_fn
        })
    }

    unsafe fn do_opt(&mut self) {
        let pass = llvm::core::LLVMCreatePassManager();
        llvm::transforms::scalar::LLVMAddConstantPropagationPass(pass);
        llvm::transforms::scalar::LLVMAddInstructionCombiningPass(pass);
        llvm::transforms::scalar::LLVMAddPromoteMemoryToRegisterPass(pass);
        llvm::transforms::scalar::LLVMAddGVNPass(pass);
        llvm::transforms::scalar::LLVMAddCFGSimplificationPass(pass);
        llvm::core::LLVMRunPassManager(pass, self.module);
        llvm::core::LLVMDisposePassManager(pass);
    }

    unsafe fn run_jit(&mut self) {
        llvm::execution_engine::LLVMRunFunction(
            self.exec_engine,
            self.brainfuck_fn,
            0,
            std::ptr::null_mut()
        );
    }
}

impl Drop for LLVMBackend {
    fn drop(&mut self) {
        unsafe {
            llvm::execution_engine::LLVMDisposeExecutionEngine(self.exec_engine);
            // llvm::core::LLVMDisposeModule(self.module);
            llvm::core::LLVMShutdown();
        }
    }
}

mod utils {
    use llvm;
    use llvm::prelude::LLVMValueRef;

    pub unsafe fn get_int8_const(c: i8) -> LLVMValueRef {
        llvm::core::LLVMConstInt(
            llvm::core::LLVMInt8Type(),
            c as _,
            false as _
        )
    }

    pub unsafe fn get_int32_const(c: isize) -> LLVMValueRef {
        llvm::core::LLVMConstInt(
            llvm::core::LLVMInt32Type(),
            c as _,
            false as _
        )
    }
}
