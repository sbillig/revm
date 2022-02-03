pub use crate::Return;

macro_rules! try_or_fail {
    ( $e:expr ) => {
        match $e {
            Ok(v) => v,
            Err(e) => return e,
        }
    };
}

macro_rules! check {
    ($expresion:expr) => {
        if !$expresion {
            return Return::NotActivated;
        }
    };
}

macro_rules! gas {
    ($machine:expr, $gas:expr) => {
        if crate::USE_GAS {
            if !$machine.gas.record_cost(($gas)) {
                return Return::OutOfGas;
            }
        }
    };
}

macro_rules! refund {
    ($machine:expr, $gas:expr) => {{
        if crate::USE_GAS {
            $machine.gas.gas_refund($gas);
        }
    }};
}

macro_rules! gas_or_fail {
    ($machine:expr, $gas:expr) => {
        if crate::USE_GAS {
            match $gas {
                Some(gas_used) => gas!($machine, gas_used),
                None => return Return::OutOfGas,
            }
        }
    };
}

macro_rules! memory_resize {
    ($machine:expr, $offset:expr, $len:expr) => {{
        let len: usize = $len;
        let offset: usize = $offset;
        if let Some(new_size) =
            crate::machine::memory::next_multiple_of_32(offset.saturating_add(len))
        {
            if new_size > $machine.memory.len() {
                if crate::USE_GAS {
                    let num_bytes = new_size / 32;
                    if !$machine
                        .gas
                        .record_memory(crate::instructions::gas::memory_gas(num_bytes))
                    {
                        return Return::OutOfGas;
                    }
                }
                $machine.memory.resize(new_size);
            }
        } else {
            return Return::OutOfGas;
        }
    }};
}

macro_rules! pop_address {
    ( $machine:expr, $x1:ident) => {
        if $machine.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        let $x1: H160 = {
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian(temp.as_bytes_mut());
            temp.into()
        };
    };
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        $x1: H160 = {
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian(temp.as_bytes_mut());
            temp.into()
        };
        $x2: H160 = {
            temp = H256::zero();
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian(temp.as_bytes_mut());
            temp.into();
        };
    };
}

macro_rules! top2 {
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let $x1 = $machine.stack.pop_unsafe();
        let $x2 = $machine.stack.top_mut().unwrap();
    };
}

macro_rules! pop {
    ( $machine:expr, $x1:ident) => {
        if $machine.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        let $x1 = $machine.stack.pop_unsafe();
    };
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let ($x1, $x2) = $machine.stack.pop2_unsafe();
    };
    ( $machine:expr, $x1:ident, $x2:ident, $x3:ident) => {
        if $machine.stack.len() < 3 {
            return Return::StackUnderflow;
        }
        let ($x1, $x2, $x3) = $machine.stack.pop3_unsafe();
    };

    ( $machine:expr, $x1:ident, $x2:ident, $x3:ident, $x4:ident) => {
        if $machine.stack.len() < 4 {
            return Return::StackUnderflow;
        }
        let ($x1, $x2, $x3, $x4) = $machine.stack.pop4_unsafe();
    };
}

macro_rules! push_h256 {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
		    match $machine.stack.push_h256($x) {
			Ok(()) => (),
			Err(e) => return e,
		    }
		)*
	)
}

macro_rules! push {
    ( $machine:expr, $( $x:expr ),* ) => (
		$(
		    match $machine.stack.push($x) {
			Ok(()) => (),
			Err(e) => return e,
		    }
		)*
	)
}

macro_rules! op1_u256_fn {
    ( $machine:expr, $op:path ) => {{
        //gas!($machine, $gas);
        if let Some(op1) = $machine.stack.top_mut() {
            *op1 = $op(*op1);
            Return::Continue
        } else {
            Return::StackUnderflow
        }
    }};
}

macro_rules! op2_u256_bool_ref {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        top2!($machine, op1, op2);
        let ret = op1.$op(op2);
        *op2 = if ret { U256::one() } else { U256::zero() };

        Return::Continue
    }};
}

macro_rules! op2_u256 {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        top2!($machine, op1, op2);
        *op2 = op1.$op(*op2);

        Return::Continue
    }};
}

macro_rules! op2_u256_tuple {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        top2!($machine, op1, op2);
        let (ret, ..) = op1.$op(*op2);
        *op2 = ret;

        Return::Continue
    }};
}

macro_rules! op2_u256_fn {
    ( $machine:expr, $op:path ) => {{
        //gas!($machine, $gas);

        top2!($machine, op1, op2);
        *op2 = $op(op1, *op2);

        Return::Continue
    }};
    ( $machine:expr, $op:path, $enabled:expr) => {{
        check!(($enabled));
        op2_u256_fn!($machine, $op)
    }};
}

macro_rules! op3_u256_fn {
    ( $machine:expr, $op:path) => {{
        //gas!($machine, $gas);

        pop!($machine, op1, op2, op3);
        let ret = $op(op1, op2, op3);
        push!($machine, ret);

        Return::Continue
    }};
    ( $machine:expr, $op:path, $spec:ident :: $enabled:ident) => {{
        check!($spec::$enabled);
        op3_u256_fn!($machine, $op)
    }};
}

macro_rules! as_usize_saturated {
    ( $v:expr ) => {{
        if { $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 } {
            usize::MAX
        } else {
            $v.0[0] as usize
        }
    }};
}

macro_rules! as_usize_or_fail {
    ( $v:expr ) => {{
        if { $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 } {
            return Return::OutOfGas;
        }

        $v.0[0] as usize
    }};

    ( $v:expr, $reason:expr ) => {{
        if { $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 } {
            return $reason;
        }

        $v.0[0] as usize
    }};
}
