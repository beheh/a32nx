use std::cmp::Ordering;
use uom::si::angle::degree;
use uom::si::length::Length;
use uom::si::Dimension;

pub trait Ssm {
    fn ssm1(&self) -> bool;
    fn ssm2(&self) -> bool;

    fn is_val(&self) -> bool {
        !self.ssm1()
    }

    fn is_inv(&self) -> bool {
        self.ssm1() && self.ssm2()
    }

    fn is_ncd(&self) -> bool {
        self.ssm1() && !self.ssm2()
    }

    fn is_ft(&self) -> bool {
        !self.ssm1() && self.ssm2()
    }
}

pub struct DiscreteParameter {
    value: bool,
    ssm1: bool,
    ssm2: bool,
}

impl DiscreteParameter {
    pub fn new(value: bool) -> Self {
        Self {
            value: value,
            ssm1: false,
            ssm2: false,
        }
    }

    pub fn new_inv(value: bool) -> Self {
        Self {
            value: value,
            ssm1: true,
            ssm2: true,
        }
    }

    pub fn as_bool(&self) -> bool {
        self.value
    }
}

impl Ssm for DiscreteParameter {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }
}

pub struct SynchroParameter {
    value: degree,
    ssm1: bool,
    ssm2: bool,
}

impl SynchroParameter {
    pub fn new_synchro(value: degree) -> Self {
        Self {
            value: value,
            ssm1: false, // not between 0 and 360
            ssm2: false,
        }
    }

    pub fn new_rvdt(value: degree) -> Self {
        Self {
            value: value,
            ssm1: false, // not between -35 and 35
            ssm2: false,
        }
    }
}

impl Ssm for SynchroParameter {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }
}

pub struct Arinc429Parameter<T> {
    value: T,
    ssm1: bool,
    ssm2: bool,
}

impl<T> Arinc429Parameter<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: value,
            ssm1: false,
            ssm2: false,
        }
    }

    pub fn new_ncd(value: T) -> Self {
        Self {
            value: value,
            ssm1: true,
            ssm2: false,
        }
    }

    pub fn new_inv(value: T) -> Self {
        Self {
            value: value,
            ssm1: true,
            ssm2: true,
        }
    }
}

impl<T> Ssm for Arinc429Parameter<T> {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }
}

pub trait AsBool {
    fn as_bool(&self) -> bool;
}

impl AsBool for Arinc429Parameter<bool> {
    fn as_bool(&self) -> bool {
        self.value
    }
}

pub trait AsF64 {
    fn as_f64(&self) -> f64;
}

impl AsF64 for Arinc429Parameter<f64> {
    fn as_f64(&self) -> f64 {
        self.value
    }
}
