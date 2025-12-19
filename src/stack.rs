use crate::{src, Decimal, DebugErr, DecimalOps, Result};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Op2 {
    Add,
    Sub,
    Mul,
    Div,
    // SwapXY,
    // ClearX,
}

/*
#[derive(Clone, Copy, Debug)]
pub enum Op1 {
    Enter(Decimal),
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Op2(Op2),
    Op1(Op1),
}
*/

const STACK_SIZE: u8 = 4;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Stack {
    Ok { data: [Decimal; STACK_SIZE as usize], stack_count: u8 },
    Err(DebugErr),
}

#[allow(dead_code)]
impl Stack {
    pub fn new() -> Self {
        Self::Ok{
            data: [Decimal::zero(); STACK_SIZE as usize],
            stack_count: 0,
        }
    }

    pub fn x(&self) -> Result<Decimal> {
        match self {
            Self::Ok { data, stack_count } if *stack_count >= 1 => Ok(data[0]),
            Self::Ok {..} => Err(src!("x not set")),
            Self::Err(e) => Err(e.clone()),
        }
    }

    pub fn y(&self) -> Result<Decimal> {
        match self {
            Self::Ok { data, stack_count } if *stack_count >= 2 => Ok(data[1]),
            Self::Ok {..} => Err(src!("x not set")),
            Self::Err(e) => Err(e.clone()),
        }
    }    

    pub fn enter(mut self, n: Decimal) -> Self {
        if let Self::Ok { data, stack_count } = &mut self {
            if *stack_count == STACK_SIZE {
                return Self::Err(src!("Exceeded stack size {}", STACK_SIZE));
            }
            // Shift right
            for i in (1..=*stack_count as usize).rev() {
                data[i] = data[i - 1];
            }
            data[0] = n;
            *stack_count += 1;
        }
        self
    }    

    pub fn op2(self, op: Op2) -> Self {
        match self {
            Self::Ok { data, stack_count } => {
                if stack_count < 2 {
                    let e = src!("y value missing");
                    return Self::Err(e)
                }
                let result_x = match op {
                    Op2::Add => data[1].add(data[0]),
                    Op2::Sub => data[1].sub(data[0]),
                    Op2::Mul => data[1].mul(data[0]),
                    Op2::Div => data[1].div(data[0]),
                };
                match result_x {
                    Ok(x) => {
                        Self::Ok {
                            data: [ x, data[2], data[3], Decimal::zero() ],
                            stack_count: stack_count - 1,
                        }
                    },
                    Err(e) => {
                        // Not sure about this yet. Might need to adjust error message.
                        Self::Err(e)
                    }
                }
            },
            Self::Err(_) => { self }
        }
    }

    #[cfg(test)]
    fn stack_count(&self) -> Option<u8> {
        match self {
            Self::Ok { stack_count, .. } => Some(*stack_count),
            _ => None,
        }
    }

    #[cfg(test)]
    fn data(&self) -> Option<[Decimal; STACK_SIZE as usize]> {
        match self {
            Self:: Ok { data, .. } => Some(*data),
            _ => None,
        }
    }

    const fn is_err(&self) -> bool {
        match self {
            Self::Ok {..} => false,
            Self::Err(_) => true,
        }
    }

    fn err(&self) -> Option<DebugErr> {
        match self {
            Self::Ok {..} => None,
            Self::Err(e) => Some(e.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use crate::rpn;
    use super::*;

    // dev dependencies
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::arbitrary::{Arbitrary, any};

    fn valid_decimal() -> impl Strategy<Value = Decimal> {
        (any::<i64>(), 0..=28u32)
            .prop_filter("valid Decimal::new", |(n, s)| Decimal::new(*n, *s).is_ok())
            .prop_map(|(num, scale)| Decimal::new(num, scale).unwrap())
    }    

    impl Arbitrary for Op2 {
        type Parameters = ();
        type Strategy = proptest::strategy::Map<proptest::strategy::BoxedStrategy<usize>, fn(usize) -> Op2>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (0..4usize).boxed().prop_map(|i| match i {
                0 => Op2::Add,
                1 => Op2::Sub,
                2 => Op2::Mul,
                _ => Op2::Div,
            })
        }
    }    

    proptest! {
        #[test]
        fn binary_add_works(a in valid_decimal(), b in valid_decimal()) {

            dbg!(&a, &b);
            let expected = a.add(b);

            let stack = Stack::new()
                .enter(a)
                .enter(b)
                .op2(Op2::Add);

            dbg!(&expected);
            dbg!(&stack);

            if expected.is_ok() {
                prop_assert_eq!(stack.x().unwrap(), expected.unwrap());
                prop_assert!(stack.y().is_err());
            } else {
                assert!(stack.x().is_err());
                assert!(stack.y().is_err());
            }
        }
    }

    // Model the stack len on the condition that stack hasn't failed.
    proptest! {
        #[test]
        fn model_stack_count(
            ops in vec((any::<Op2>(), valid_decimal(), valid_decimal()), 0..5)
        ) {
            // Model the number of data items on the stack.
            #[derive(Debug, Clone)]
            struct Model { stack_count: isize }

            impl Model {
                fn new() -> Self { Model { stack_count: 0 } }

                // Assume stack will fail if stack_count exceeds STACK_LEN
                fn enter(self, _n: Decimal) -> Self { Model { stack_count: self.stack_count + 1 } }

                // Assume stack will fail on negative len
                fn op2(self, op: Op2) -> Self {
                    Model { stack_count: self.stack_count - 1 }
                }

                fn stack_count(&self) -> isize { self.stack_count }
            }

            let mut model = Model::new();
            let mut stack = Stack::new();

            // Have to fix this. It must be more general than this.
            for (op, a, b) in ops {
                model = model.enter(a).enter(b).op2(op);
                stack = stack.enter(a).enter(b).op2(op);
            }

            // We only test under the condition that the stack is not in an error state.
            if !stack.is_err() {
                prop_assert_eq!(model.stack_count(), stack.stack_count().unwrap() as isize);
            }
        }
    }

    #[test]
    pub fn rpn_macro_works1() {

        let x = dec!(1.0);
        let stack: Stack = rpn!(x);
        assert_eq!(stack.x().unwrap(), x);
    }

    #[test]
    pub fn rpn_macro_works2() {
        let x = dec!(1.0);
        let y = dec!(2.0);
        let stack: Stack = rpn!(x, y, +);
        assert_eq!(stack.x().unwrap(), dec!(3.0));
    }
}
