#![allow(unused)]

#[cfg(test)]
mod tests {
    use anyhow::*;
    use chibiwasm::execution::runtime::{Exports, Runtime};
    use chibiwasm::execution::value::Value;
    use log::debug;
    use paste::paste;
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::io::{Cursor, Read};
    use std::rc::Rc;
    use std::sync::Once;
    use std::{fs, path::Path};
    use wabt::{script::*, Features};

    static INIT: Once = Once::new();

    #[derive(Debug, Default)]
    struct Spec {
        modules: HashMap<Option<String>, Rc<RefCell<Runtime>>>,
    }

    fn into_wasm_value(values: Vec<wabt::script::Value>) -> Vec<Value> {
        values
            .into_iter()
            .map(|a| match a {
                wabt::script::Value::I32(v) => Value::I32(v),
                wabt::script::Value::I64(v) => Value::I64(v),
                wabt::script::Value::F32(v) => Value::F32(v),
                wabt::script::Value::F64(v) => Value::F64(v),
                wabt::script::Value::V128(_) => todo!(),
            })
            .collect()
    }

    fn run_test(spec_file: &str) -> Result<()> {
        INIT.call_once(|| {
            let _ = pretty_env_logger::env_logger::builder()
                .is_test(true)
                .try_init();
        });

        let spec = Path::new("./tests/testsuite").join(spec_file);
        let mut file = fs::File::open(spec)?;
        let mut wast = String::new();
        file.read_to_string(&mut wast)?;

        let features = {
            let mut f = Features::new();
            f.enable_all();
            f
        };
        let mut parser = ScriptParser::<f32, f64>::from_source_and_name_with_features(
            wast.as_bytes(),
            spec_file,
            features,
        )?;

        let spec = &mut Spec {
            modules: HashMap::new(),
        };

        fn assert_values(results: Vec<Value>, expected: Vec<wabt::script::Value>) -> Result<()> {
            let got: Vec<_> = results
                .into_iter()
                .map(|result| match result {
                    Value::I32(v) => wabt::script::Value::I32(v),
                    Value::I64(v) => wabt::script::Value::I64(v),
                    Value::F32(v) => {
                        if v.is_nan() {
                            wabt::script::Value::F32(0_f32)
                        } else {
                            wabt::script::Value::F32(v)
                        }
                    }
                    Value::F64(v) => {
                        if v.is_nan() {
                            wabt::script::Value::F64(0_f64)
                        } else {
                            wabt::script::Value::F64(v)
                        }
                    }
                })
                .collect();

            let want: Vec<_> = expected
                .into_iter()
                .map(|e| match e {
                    wabt::script::Value::F32(v) => {
                        if v.is_nan() {
                            return wabt::script::Value::F32(0_f32);
                        }
                        e
                    }
                    wabt::script::Value::F64(v) => {
                        if v.is_nan() {
                            return wabt::script::Value::F64(0_f64);
                        }
                        e
                    }
                    _ => e,
                })
                .collect();
            assert_eq!(want, got, "unexpect result, want={want:?}, got={got:?}");
            Ok(())
        }

        fn invoke(
            runtime: &mut Runtime,
            field: String,
            args: Vec<wabt::script::Value>,
            expected: Vec<wabt::script::Value>,
        ) -> Result<()> {
            let args = into_wasm_value(args);
            let result = runtime.call(field.clone(), args.clone())?;
            match result {
                Some(result) => assert_values(vec![result], expected),
                None => Ok(()),
            }
        }

        while let Some(command) = parser.next()? {
            match command.kind {
                CommandKind::AssertReturn { action, expected } => match action {
                    Action::Invoke {
                        field,
                        args,
                        module,
                    } => {
                        debug!(
                            "invoke module: {:?}, func: {}, args: {:#?}",
                            &module, &field, &args
                        );
                        let runtime = spec.modules.get(&module).expect("not found mdoule").clone();
                        let runtime = &mut *runtime.borrow_mut();
                        invoke(runtime, field, args, expected)?;
                    }
                    Action::Get { module, field } => {
                        debug!("get module: {:?}, field: {}", &module, &field);
                        let runtime = spec.modules.get(&module).expect("not found mdoule").clone();
                        let runtime = &mut *runtime.borrow_mut();
                        let exports = runtime.exports(field.clone())?;

                        let results = match exports {
                            Exports::Global(global) => vec![global.value.clone()],
                            _ => {
                                todo!();
                            }
                        };

                        assert_values(results, expected);
                    }
                },
                CommandKind::PerformAction(action) => match action {
                    Action::Invoke {
                        field,
                        args,
                        module,
                    } => {
                        debug!(
                            "invoke module: {:?}, func: {}, args: {:#?}",
                            &module, &field, &args
                        );
                        let runtime = spec.modules.get(&module).expect("not found mdoule").clone();
                        let runtime = &mut *runtime.borrow_mut();
                        invoke(runtime, field, args, vec![])?;
                    }
                    Action::Get { .. } => todo!(),
                },
                CommandKind::AssertReturnCanonicalNan { .. } => {
                    // TODO
                }
                CommandKind::AssertReturnArithmeticNan { .. } => {
                    // TODO
                }
                CommandKind::AssertTrap { action, message } => match action {
                    Action::Invoke {
                        field,
                        args,
                        module,
                    } => {
                        debug!(
                            "invoke module: {:?}, func: {}, args: {:#?}",
                            &module, &field, &args
                        );
                        let runtime = spec.modules.get(&module).expect("not found mdoule").clone();
                        let runtime = &mut *runtime.borrow_mut();
                        let args = into_wasm_value(args);
                        let result = runtime.call(field.clone(), args.clone());

                        match result {
                            Err(err) => {
                                let want = message;
                                let got = err.to_string();
                                assert_eq!(
                                    want,
                                    got,
                                    "unexpect result, want={want}, got={got}, test: {field}, args: {args:?}",
                                );
                            }
                            _ => {
                                panic!("test must be fail: {}", field);
                            }
                        }
                    }
                    Action::Get { .. } => todo!(),
                },
                CommandKind::AssertInvalid { .. } => {
                    // TODO
                }
                CommandKind::AssertMalformed { .. } => {
                    // TODO
                }
                CommandKind::AssertUninstantiable { .. } => {
                    // TODO
                }
                CommandKind::AssertExhaustion { .. } => {
                    // TODO
                }
                CommandKind::AssertUnlinkable { .. } => {
                    // TODO
                }
                CommandKind::Register { .. } => {
                    // TODO
                }
                CommandKind::Module { module, name } => {
                    let mut reader = Cursor::new(module.into_vec());
                    let runtime = Runtime::from_reader(&mut reader)?;
                    let runtime = Rc::new(RefCell::new(runtime));
                    spec.modules.insert(name, runtime.clone());
                    spec.modules.insert(None, runtime);
                }
                _ => {
                    panic!("unexpect command kind: {:?}", command.kind);
                }
            }
        }
        Ok(())
    }

    macro_rules! test {
        ($ty: ident) => {
            paste! {
                #[test]
                fn [<test_ $ty>]() -> Result<()> {
                    let file = format!("{}.wast", stringify!($ty));
                    run_test(&file)?;
                    Ok(())
                }
            }
        };
    }

    test!(i32);
    test!(i64);
    test!(f32);
    test!(f32_cmp);
    test!(f32_bitwise);
    test!(f64);
    test!(f64_cmp);
    test!(f64_bitwise);
    test!(load);
    test!(nop);
    test!(store);
    test!(loop);
    test!(int_literals);
    test!(if);
    test!(br_if);
    test!(globals);
    test!(func);
    test!(block);
    test!(comments);
    test!(binary);
    test!(break_drop);
    test!(const);
    test!(forward);
    test!(inline_module);
    test!(names);
    test!(stack);
    test!(return);
    test!(br);
    test!(br_table);
    test!(local_set);
    test!(local_get);
    test!(local_tee);
    test!(select);
    test!(labels);
    test!(unreachable);
    test!(type);
    test!(fac);
    test!(memory_size);
    test!(address);
    test!(memory_trap);
    test!(align);
    test!(memory);
    test!(float_misc);
    test!(int_exprs);
    test!(memory_grow);
    test!(memory_redundancy);
    // NOTE: this will overflow in the test thread, so we need use RUST_MIN_STACK=104857600 to run this test
    test!(call);
    test!(call_indirect);
    test!(float_memory);
    test!(float_exprs);
    test!(left_to_right);
    test!(skip_stack_guard_page);
    test!(unwind);
    test!(binary_leb128);
    test!(exports);

    //test!(linking);
    //test!(conversions); // cannot parse
    //test!(start);
    //test!(imports);
    //test!(func_ptrs);
    //test!(float_literals);
    //test!(elem);
    //test!(endianness);
    //test!(data);
    test!(switch);
    //test!(token);
    //test!(traps);
    //test!(unreached_invalid);
    //test!(utf8_custom_section_id);
    //test!(utf8_import_field);
    //test!(utf8_import_module);
    //test!(utf8_invalid_encoding);
}
