use uom::si::angle::degree;

pub trait Value<T> {
    fn value(&self) -> T;
}

/// This trait allows access to the Sign/Status Matrix (SSM). It is a two-bit matrix in one of four
/// states:
/// | ssm1 | ssm2 | methods        | meaning                 |
/// | 0    | 0    | is_no          | valid, normal operation |
/// | 0    | 1    | is_no, is_ft   | valid, functional test  |
/// | 1    | 0    | is_ncd         | no computed data        |
/// | 1    | 1    | is_inv         | failure warning
pub trait SignStatusMatrix {
    fn ssm1(&self) -> bool;
    fn ssm2(&self) -> bool;

    fn is_ncd(&self) -> bool;
    fn is_no(&self) -> bool;
    fn is_ft(&self) -> bool;
    fn is_fw(&self) -> bool;
}

pub trait FwcSsm: SignStatusMatrix {
    fn parity(&self) -> bool;

    fn is_val(&self) -> bool {
        !self.is_fw() && self.parity()
    }
    fn is_inv(&self) -> bool {
        self.is_fw() || !self.parity()
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
}

impl SignStatusMatrix for DiscreteParameter {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }

    fn is_ncd(&self) -> bool {
        self.ssm1() && !self.ssm2()
    }
    fn is_no(&self) -> bool {
        !self.ssm1() && !self.ssm2()
    }
    fn is_ft(&self) -> bool {
        !self.ssm1() && self.ssm2()
    }
    fn is_fw(&self) -> bool {
        self.ssm1() && self.ssm2()
    }
}

impl FwcSsm for DiscreteParameter {
    fn parity(&self) -> bool {
        true
    }
}

impl Value<bool> for DiscreteParameter {
    fn value(&self) -> bool {
        self.value
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
            ssm1: true, // not between 0 and 360
            ssm2: true,
        }
    }

    pub fn new_rvdt(value: degree) -> Self {
        Self {
            value: value,
            ssm1: true, // not between -35 and 35
            ssm2: true,
        }
    }
}

impl SignStatusMatrix for SynchroParameter {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }

    fn is_ncd(&self) -> bool {
        self.ssm1() && !self.ssm2()
    }
    fn is_no(&self) -> bool {
        self.ssm1() && self.ssm2()
    }
    fn is_ft(&self) -> bool {
        !self.ssm1() && self.ssm2()
    }
    fn is_fw(&self) -> bool {
        !self.ssm1() && !self.ssm2()
    }
}

impl FwcSsm for SynchroParameter {
    fn parity(&self) -> bool {
        true
    }
}

impl Value<degree> for SynchroParameter {
    fn value(&self) -> degree {
        self.value
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
            ssm1: true,
            ssm2: true,
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
            ssm1: false,
            ssm2: false,
        }
    }
}

impl<T> SignStatusMatrix for Arinc429Parameter<T> {
    fn ssm1(&self) -> bool {
        self.ssm1
    }
    fn ssm2(&self) -> bool {
        self.ssm2
    }

    fn is_ncd(&self) -> bool {
        self.ssm1() && !self.ssm2()
    }
    fn is_no(&self) -> bool {
        self.ssm1() && self.ssm2()
    }
    fn is_ft(&self) -> bool {
        !self.ssm1() && self.ssm2()
    }
    fn is_fw(&self) -> bool {
        !self.ssm1() && !self.ssm2()
    }
}

impl<T> FwcSsm for Arinc429Parameter<T> {
    fn parity(&self) -> bool {
        true
    }
}

impl<T> Value<T> for Arinc429Parameter<T>
where
    T: Copy,
{
    fn value(&self) -> T {
        self.value
    }
}
