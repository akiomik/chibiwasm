use crate::error::Error;
use crate::{runtime::Runtime, value::Value};
use anyhow::{bail, Context, Result};
use num_derive::FromPrimitive;

// https://webassembly.github.io/spec/core/binary/instructions.html#expressions
#[derive(Debug, FromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Unreachable = 0x00,
    Nop = 0x01,
    LocalGet = 0x20,
    Call = 0x10,
    I32Const = 0x41,
    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,
    I32Add = 0x6a,
    I32Sub = 0x6b,
    I32Mul = 0x6c,
    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32Popcnt = 0x69,
    I32DivS = 0x6D,
    I32DivU = 0x6E,
    I32RemS = 0x6F,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32ShL = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32RtoL = 0x77,
    I32RtoR = 0x78,
    I32Extend8S = 0xC0,
    I32Extend16S = 0xC1,
    I64Const = 0x42,
    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,
    I64Clz = 0x79,
    I64Ctz = 0x7A,
    I64Popcnt = 0x7B,
    I64Add = 0x7C,
    I64Sub = 0x7D,
    I64Mul = 0x7E,
    I64DivS = 0x7F,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64ShL = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64RtoL = 0x89,
    I64RtoR = 0x8A,
    I64Extend8S = 0xC2,
    I64Extend16S = 0xC3,
    I64Extend32S = 0xC4,
    F32Const = 0x43,
    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,
    F32Abs = 0x8B,
    F32Neg = 0x8C,
    F32Ceil = 0x8D,
    F32Floor = 0x8E,
    F32Trunc = 0x8F,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,
    F32Copysign = 0x98,
    Return = 0x0f,
    If = 0x04,
    Else = 0x05,
    End = 0x0b,
    Void = 0x40,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Unreachable,
    Nop,
    LocalGet(u32),
    Call(u32),
    I32Const(i32),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32ShL,
    I32ShrS,
    I32ShrU,
    I32RtoL,
    I32RtoR,
    I32Extend8S,
    I32Extend16S,
    I64Const(i64),
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64ShL,
    I64ShrS,
    I64ShrU,
    I64RtoL,
    I64RtoR,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,
    F32Const(f32),
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    Return,
    If,
    Else,
    End,
    Void,
}

pub fn pop_rl(runtime: &mut Runtime) -> Result<(Value, Value)> {
    let r = runtime.stack.pop().ok_or_else(|| Error::StackPopError)?;
    let l = runtime.stack.pop().ok_or_else(|| Error::StackPopError)?;
    Ok((r, l))
}

pub fn local_get(runtime: &mut Runtime, idx: usize) -> Result<()> {
    let value = runtime
        .current_frame()?
        .local_stack
        .get(idx)
        .context("not found local variable")?;
    runtime.stack.push(value.clone());
    Ok(())
}

pub fn popcnt(runtime: &mut Runtime) -> Result<()> {
    let value = runtime.stack_pop()?;
    match value {
        Value::I32(v) => runtime.stack.push(v.count_ones().into()),
        Value::I64(v) => runtime.stack.push((v.count_ones() as i64).into()),
        _ => bail!("unexpected value"),
    }
    Ok(())
}

pub fn i32const(runtime: &mut Runtime, value: i32) -> Result<()> {
    runtime.stack.push(value.into());
    Ok(())
}

pub fn push<T: Into<Value>>(runtime: &mut Runtime, value: T) -> Result<()> {
    runtime.stack.push(value.into());
    Ok(())
}

pub fn i64extend_32s(runtime: &mut Runtime) -> Result<()> {
    let value = runtime.stack.pop().ok_or_else(|| Error::StackPopError)?;
    match value {
        Value::I64(v) => {
            let result = v << 32 >> 32;
            runtime.stack.push(result.into());
        }
        _ => bail!("unexpected value type"),
    }
    Ok(())
}

macro_rules! impl_binary_operation {
    ($($op: ident),*) => {
        $(
            pub fn $op(runtime: &mut Runtime) -> Result<()> {
                let (r, l) = pop_rl(runtime)?;
                runtime.stack.push(l.$op(&r)?);
                Ok(())
            }
        )*
    };
}

macro_rules! impl_unary_operation {
    ($($op: ident),*) => {
        $(
            pub fn $op(runtime: &mut Runtime) -> Result<()> {
                let value = runtime.stack.pop().ok_or_else(|| Error::StackPopError)?;
                runtime.stack.push(value.$op()?);
                Ok(())
            }
         )*
    };
}

impl_unary_operation!(
    eqz, // itestop
    clz, ctz, extend8_s, extend16_s, // iunop
    abs, neg, sqrt, ceil, floor, trunc, nearest // funop
);
impl_binary_operation!(
    add, sub, mul, // binop
    div_s, div_u, rem_s, rem_u, and, or, xor, shl, shr_u, shr_s, rotl, rotr, // ibinop
    min, max, div, // fbinop
    equal, not_equal, // relop
    lt_s, lt_u, gt_s, gt_u, le_s, le_u, ge_s, ge_u, // irelop
    flt, fgt, fle, fge // frelop
);
