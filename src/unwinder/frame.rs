use gimli::{
    BaseAddresses, CfaRule, EvaluationResult, Expression, Location, RegisterRule,
    UninitializedUnwindContext, UnwindTableRow, Value,
};

use super::find_fde::{self, FDEFinder, FDESearchResult};
use crate::abi::PersonalityRoutine;
use crate::arch::*;
use super::arch::*;
use crate::util::*;

#[derive(Debug)]
pub struct Frame {
    fde_result: FDESearchResult,
    row: UnwindTableRow<StaticSlice>,
}

impl Frame {
    pub fn from_context(ctx: &Context) -> Result<Option<Self>, gimli::Error> {
        let mut ra = ctx[Arch::RA];

        // Reached end of stack
        if ra == 0 {
            return Ok(None);
        }

        // RA points to the *next* instruction, so move it back 1 byte for the call instruction.
        ra -= 1;

        let fde_result = match find_fde::get_finder().find_fde(ra as _) {
            Some(v) => v,
            None => return Ok(None),
        };
        let mut unwinder = UninitializedUnwindContext::new();
        let row = fde_result
            .fde
            .unwind_info_for_address(
                &fde_result.eh_frame,
                &fde_result.bases,
                &mut unwinder,
                ra as _,
            )?
            .clone();

        Ok(Some(Self { fde_result, row }))
    }

    #[cfg(feature = "dwarf-expr")]
    fn evaluate_expression(
        &self,
        ctx: &Context,
        expr: Expression<StaticSlice>,
    ) -> Result<usize, gimli::Error> {
        let mut eval = expr.evaluation(self.fde_result.fde.cie().encoding());
        let mut result = eval.evaluate()?;
        loop {
            match result {
                EvaluationResult::Complete => break,
                EvaluationResult::RequiresMemory { address, .. } => {
                    let value = unsafe { (address as usize as *const usize).read_unaligned() };
                    result = eval.resume_with_memory(Value::Generic(value as _))?;
                }
                EvaluationResult::RequiresRegister { register, .. } => {
                    let value = ctx[register];
                    result = eval.resume_with_register(Value::Generic(value as _))?;
                }
                EvaluationResult::RequiresRelocatedAddress(address) => {
                    let value = unsafe { (address as usize as *const usize).read_unaligned() };
                    result = eval.resume_with_memory(Value::Generic(value as _))?;
                }
                _ => unreachable!(),
            }
        }

        Ok(
            match eval
                .result()
                .pop()
                .ok_or(gimli::Error::PopWithEmptyStack)?
                .location
            {
                Location::Address { address } => address as usize,
                _ => unreachable!(),
            },
        )
    }

    #[cfg(not(feature = "dwarf-expr"))]
    fn evaluate_expression(
        &self,
        ctx: &Context,
        expr: Expression<StaticSlice>,
    ) -> Result<usize, gimli::Error> {
        Err(gimli::Error::UnsupportedEvaluation)
    }

    pub fn unwind(&self, ctx: &Context) -> Result<Context, gimli::Error> {
        let row = &self.row;
        let mut new_ctx = ctx.clone();

        let cfa = match *row.cfa() {
            CfaRule::RegisterAndOffset { register, offset } => {
                ctx[register].wrapping_add(offset as usize)
            }
            CfaRule::Expression(expr) => self.evaluate_expression(ctx, expr)?,
        };

        new_ctx[Arch::SP] = cfa as _;
        new_ctx[Arch::RA] = 0;

        for (reg, rule) in row.registers() {
            let value = match *rule {
                RegisterRule::Undefined | RegisterRule::SameValue => ctx[*reg],
                RegisterRule::Offset(offset) => unsafe {
                    *((cfa.wrapping_add(offset as usize)) as *const usize)
                },
                RegisterRule::ValOffset(offset) => cfa.wrapping_add(offset as usize),
                RegisterRule::Register(r) => ctx[r],
                RegisterRule::Expression(expr) => {
                    let addr = self.evaluate_expression(ctx, expr)?;
                    unsafe { *(addr as *const usize) }
                }
                RegisterRule::ValExpression(expr) => self.evaluate_expression(ctx, expr)?,
                RegisterRule::Architectural => unreachable!(),
            };
            new_ctx[*reg] = value;
        }

        Ok(new_ctx)
    }

    pub fn bases(&self) -> &BaseAddresses {
        &self.fde_result.bases
    }

    pub fn personality(&self) -> Option<PersonalityRoutine> {
        self.fde_result
            .fde
            .personality()
            .map(|x| unsafe { deref_pointer(x) })
            .map(|x| unsafe { core::mem::transmute(x) })
    }

    pub fn lsda(&self) -> usize {
        self.fde_result
            .fde
            .lsda()
            .map(|x| unsafe { deref_pointer(x) })
            .unwrap_or(0)
    }

    pub fn initial_address(&self) -> usize {
        self.fde_result.fde.initial_address() as _
    }
}
