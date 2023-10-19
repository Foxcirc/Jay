

use std::{fmt, collections::HashMap};
use crate::{parser::{Literal, OpKind, Op, Span, IdentStr}, diagnostic::Diagnostic, parse_modules::SourceFile, arch::Intrinsic};

pub(crate) fn codegen<I: Intrinsic>(program: &mut Program<I>, source_file: SourceFile) -> Result<(), CodegenError> {

    // let file_name = source_file.name().to_string();

    for fun in source_file.items.funs {

        let mut state = State::<I>::default();
        codegen_block(&mut state, fun.body, None)?;
        state.bytecode.push(Instr::spanned(InstrKind::Return, fun.span));

        program.funs.insert(fun.name, state.bytecode);
    
    }

    Ok(())

}

fn codegen_block<I: Intrinsic>(state: &mut State<I>, block: Vec<Op>, loop_escape: Option<Label>) -> Result<(), CodegenError> {

    for op in block {

        match op.kind {

            OpKind::Push { value } => state.bytecode.push(Instr::spanned(InstrKind::Push { value }, op.span)),

            OpKind::Call { name }  => {
                let inrinsic = I::generate(&name);
                let result = match inrinsic {
                    Some(val) => InstrKind::Intrinsic(val),
                    None => InstrKind::Call { to: name }
                };
                state.bytecode.push(Instr::spanned(result, op.span));
            },

            OpKind::Copy  => state.bytecode.push(Instr::spanned(InstrKind::Copy, op.span)),
            OpKind::Over  => state.bytecode.push(Instr::spanned(InstrKind::Over, op.span)),
            OpKind::Swap  => state.bytecode.push(Instr::spanned(InstrKind::Swap, op.span)),
            OpKind::Rot3  => state.bytecode.push(Instr::spanned(InstrKind::Rot3, op.span)),
            OpKind::Rot4  => state.bytecode.push(Instr::spanned(InstrKind::Rot4, op.span)),
            OpKind::Drop  => state.bytecode.push(Instr::spanned(InstrKind::Drop, op.span)),

            OpKind::Read  => state.bytecode.push(Instr::spanned(InstrKind::Read,  op.span)),
            OpKind::Write => state.bytecode.push(Instr::spanned(InstrKind::Write, op.span)),

            OpKind::Move  => todo!(),
            OpKind::Addr  => todo!(),
            OpKind::Type  => todo!(),
            OpKind::Size  => todo!(),
            OpKind::Dot   => todo!(),

            OpKind::Add => state.bytecode.push(Instr::spanned(InstrKind::Add, op.span)),
            OpKind::Sub => state.bytecode.push(Instr::spanned(InstrKind::Sub, op.span)),
            OpKind::Mul => state.bytecode.push(Instr::spanned(InstrKind::Mul, op.span)),
            OpKind::Dvm => state.bytecode.push(Instr::spanned(InstrKind::Dvm, op.span)),

            OpKind::Not => state.bytecode.push(Instr::spanned(InstrKind::Not, op.span)),
            OpKind::And => state.bytecode.push(Instr::spanned(InstrKind::And, op.span)),
            OpKind::Or  => state.bytecode.push(Instr::spanned(InstrKind::Or,  op.span)),
            OpKind::Xor => state.bytecode.push(Instr::spanned(InstrKind::Xor, op.span)),

            OpKind::Eq  => state.bytecode.push(Instr::spanned(InstrKind::Eq,  op.span)),
            OpKind::Gt  => state.bytecode.push(Instr::spanned(InstrKind::Gt,  op.span)),
            OpKind::Gte => state.bytecode.push(Instr::spanned(InstrKind::Gte, op.span)),
            OpKind::Lt  => state.bytecode.push(Instr::spanned(InstrKind::Lt,  op.span)),
            OpKind::Lte => state.bytecode.push(Instr::spanned(InstrKind::Lte, op.span)),

            OpKind::If { block: if_block } => {

                let end = state.next_label();
                state.bytecode.push(Instr::spanned(InstrKind::Bne { to: end }, op.span));
                codegen_block(state, if_block, loop_escape)?;
                state.bytecode.push(Instr::spanned(InstrKind::Label { label: end, producer: Producer::If }, op.span));

            },
            OpKind::Elif { block: _block } => todo!(),
            OpKind::Else { block: else_block } => {

                // check that this `else` is coming directly after an `if`
                if !matches!(state.bytecode.last().map(|val| &val.kind), Some(InstrKind::Label { producer: Producer::If, .. })) {
                    return Err(CodegenError::spanned(CodegenErrorKind::InvalidElse, op.span))
                }

                let end = state.next_label();
                state.bytecode.insert(state.bytecode.len() - 1, Instr::spanned(InstrKind::Bra { to: end }, op.span));
                codegen_block(state, else_block, loop_escape)?;
                state.bytecode.push(Instr::spanned(InstrKind::Label { label: end, producer: Producer::Else }, op.span));

            },
            OpKind::Loop { block: loop_block } => {

                let start = state.next_label();
                let escape = state.next_label();
                state.bytecode.push(Instr::spanned(InstrKind::Label { label: start, producer: Producer::Loop }, op.span));
                codegen_block(state, loop_block, Some(escape))?;
                state.bytecode.push(Instr::spanned(InstrKind::Bra { to: start }, op.span));
                state.bytecode.push(Instr::spanned(InstrKind::Label { label: escape, producer: Producer::Break }, op.span));

            },
            OpKind::For  { block: _block } => todo!(),
            OpKind::Break => {

                let escape = match loop_escape {
                    Some(val) => val,
                    None => return Err(CodegenError::spanned(CodegenErrorKind::InvalidBreak, op.span)),
                };
                state.bytecode.push(Instr::spanned(InstrKind::Bra { to: escape }, op.span))

            },
        };

    };

    Ok(())

}

#[derive(PartialEq, Eq)]
pub(crate) struct Instr<I> {
    pub(crate) kind: InstrKind<I>,
    pub(crate) span: Span,
}

impl<I: fmt::Debug> fmt::Debug for Instr<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl<I> Instr<I> {
    pub(crate) fn spanned(kind: InstrKind<I>, span: Span) -> Self {
        Self { kind, span }
    }
    pub(crate) fn unspanned(kind: InstrKind<I>) -> Self {
        Self { kind, span: Span::default() }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InstrKind<I> {

    Label { label: Label, producer: Producer }, // todo: rename?

    Push { value: Literal },

    Call { to: IdentStr },
    Return,

    Drop,
    Copy,
    Over,
    Swap,
    Rot3,
    Rot4,

    Read,
    Write,
    // Move,

    // Addr,
    // Type,
    // Size,

    // Access,

    Add,
    Sub,
    Mul,
    Dvm,

    Not,
    And,
    Or,
    Xor,

    Eq,
    Gt,
    Gte,
    Lt,
    Lte,

    Bne { to: Label }, // todo: make it all tuple variants
    Bra { to: Label },

    Intrinsic(I),

}

pub(crate) type Label = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Producer {
    If,
    Else,
    Loop,
    Break,
}

struct State<I> {
    pub bytecode: Vec<Instr<I>>,
    pub counter: usize,
}

impl<I> Default for State<I> {
    fn default() -> Self {
        Self {
            bytecode: Vec::new(),
            counter: 0
        }
    }
}

impl<I> State<I> {
    pub fn next_label(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

pub(crate) struct Program<I> {
    pub funs: HashMap<IdentStr, Bytecode<I>>,
}

impl<I> Default for Program<I> {
    fn default() -> Self {
        Self {
            funs: HashMap::new(),
        }
    }
}

pub(crate) type Bytecode<I> = Vec<Instr<I>>;
pub(crate) type BytecodeSlice<I> = [Instr<I>];

#[derive(Debug, Default)]
pub(crate) struct FileSpan {
    pub span: Span,
    pub file: String,
}

pub(crate) struct CodegenError {
    pub(crate) kind: CodegenErrorKind,
    pub(crate) span: Span,
}

impl CodegenError {
    pub(crate) fn spanned(kind: CodegenErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub(crate) enum CodegenErrorKind {
    InvalidElse,
    InvalidBreak,
    InvalidSignature,
}

pub(crate) fn format_error(value: CodegenError) -> Diagnostic {
    let diag = match &value.kind {
        CodegenErrorKind::InvalidElse => Diagnostic::error("`else` block withput `if` block"),
        CodegenErrorKind::InvalidBreak => Diagnostic::error("`break` outside loop"),
        CodegenErrorKind::InvalidSignature => Diagnostic::error("invalid type in fn signature"),
    };
    diag.pos(value.span.to_pos())
}

