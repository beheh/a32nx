use std::time::Duration;

use systems::flight_warning::logic::{
    ConfirmationNode, MemoryNode, MonostableTriggerNode, PreceedingValueNode,
    TransientDetectionNode,
};
use systems::flight_warning::parameters::{
    Arinc429Parameter, AsBool, AsF64, DiscreteParameter, Ssm,
};
use systems::simulation::UpdateContext;
use uom::si::f64::*;

trait LhLgCompressed {
    fn lh_lg_compressed(&self, index: usize) -> &Arinc429Parameter<bool>;
}

trait EssLhLgCompressed {
    fn ess_lh_lg_compressed(&self) -> &DiscreteParameter;
}

trait NormLhLgCompressed {
    fn norm_lh_lg_compressed(&self) -> &DiscreteParameter;
}

trait RadioHeight {
    fn radio_height(&self, index: usize) -> &Arinc429Parameter<f64>;
}

trait ComputedSpeed {
    fn computed_speed(&self, index: usize) -> &Arinc429Parameter<f64>;
}

trait Eng1MasterLevelSelectOn {
    fn eng1_master_lever_select_on(&self) -> &Arinc429Parameter<bool>;
}

trait Eng2MasterLevelSelectOn {
    fn eng2_master_lever_select_on(&self) -> &Arinc429Parameter<bool>;
}

trait Eng1CoreSpeedAtOrAboveIdle {
    fn eng1_core_speed_at_or_above_idle(&self, index: usize) -> &Arinc429Parameter<bool>;
}

trait Eng2CoreSpeedAtOrAboveIdle {
    fn eng2_core_speed_at_or_above_idle(&self, index: usize) -> &Arinc429Parameter<bool>;
}

trait Eng1FirePbOut {
    fn eng_1_fire_pb_out(&self) -> &DiscreteParameter;
}

trait ToConfigTest {
    fn to_config_test(&self) -> &Arinc429Parameter<bool>;
}

struct SignalTable {
    lh_lg_compressed_1: Arinc429Parameter<bool>,
    lh_lg_compressed_2: Arinc429Parameter<bool>,
    ess_lh_lg_compressed: DiscreteParameter,
    norm_lh_lg_compressed: DiscreteParameter,
    radio_height_1: Arinc429Parameter<f64>,
    radio_height_2: Arinc429Parameter<f64>,
    computed_speed_1: Arinc429Parameter<f64>,
    computed_speed_2: Arinc429Parameter<f64>,
    computed_speed_3: Arinc429Parameter<f64>,
}

impl LhLgCompressed for SignalTable {
    fn lh_lg_compressed(&self, index: usize) -> &Arinc429Parameter<bool> {
        match index {
            1 => &self.lh_lg_compressed_1,
            2 => &self.lh_lg_compressed_2,
            _ => panic!(),
        }
    }
}

impl EssLhLgCompressed for SignalTable {
    fn ess_lh_lg_compressed(&self) -> &DiscreteParameter {
        &self.ess_lh_lg_compressed
    }
}

impl NormLhLgCompressed for SignalTable {
    fn norm_lh_lg_compressed(&self) -> &DiscreteParameter {
        &self.norm_lh_lg_compressed
    }
}

impl RadioHeight for SignalTable {
    fn radio_height(&self, index: usize) -> &Arinc429Parameter<f64> {
        match index {
            1 => &self.radio_height_1,
            2 => &self.radio_height_2,
            _ => panic!(),
        }
    }
}

impl ComputedSpeed for SignalTable {
    fn computed_speed(&self, index: usize) -> &Arinc429Parameter<f64> {
        match index {
            1 => &self.computed_speed_1,
            2 => &self.computed_speed_2,
            3 => &self.computed_speed_3,
            _ => panic!(),
        }
    }
}

trait LgciuGroundDetection {
    fn new_ground(&self) -> bool;
    fn lgciu_12_inv(&self) -> bool;
}

struct NewGroundDef {
    conf1: ConfirmationNode,
    conf2: ConfirmationNode,
    conf3: ConfirmationNode,
    conf4: ConfirmationNode,
    memory1: MemoryNode,
    memory2: MemoryNode,
    new_ground: bool,
    lgciu_12_inv: bool,
}

impl NewGroundDef {
    fn new() -> Self {
        NewGroundDef {
            conf1: ConfirmationNode::new_leading(Duration::from_secs_f64(1.0)),
            conf2: ConfirmationNode::new_leading(Duration::from_secs_f64(0.5)),
            conf3: ConfirmationNode::new_leading(Duration::from_secs_f64(1.0)),
            conf4: ConfirmationNode::new_leading(Duration::from_secs_f64(0.5)),
            memory1: MemoryNode::new(true),
            memory2: MemoryNode::new(true),
            new_ground: false,
            lgciu_12_inv: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        signals: &(impl LhLgCompressed + EssLhLgCompressed + NormLhLgCompressed),
    ) {
        let xor1 = signals.lh_lg_compressed(1).as_bool() ^ signals.ess_lh_lg_compressed().as_bool();
        let set_memory1 = signals.lh_lg_compressed(1).is_ncd()
            || signals.lh_lg_compressed(1).is_inv()
            || self.conf1.update(context, xor1);

        let memory1_out =
            self.memory1
                .update(context, set_memory1, self.conf2.update(context, !xor1));

        let xor3 =
            signals.lh_lg_compressed(2).as_bool() ^ signals.norm_lh_lg_compressed().as_bool();
        let set_memory2 = signals.lh_lg_compressed(2).is_ncd()
            || signals.lh_lg_compressed(2).is_inv()
            || self.conf3.update(context, xor3);
        let memory2_out =
            self.memory2
                .update(context, set_memory2, self.conf4.update(context, !xor3));

        let op1 = signals.lh_lg_compressed(1).as_bool() && signals.ess_lh_lg_compressed().as_bool();
        let op2 =
            signals.lh_lg_compressed(2).as_bool() && signals.norm_lh_lg_compressed().as_bool();

        self.new_ground = op1 && op2;
        self.lgciu_12_inv = memory1_out || memory2_out;
    }
}

impl LgciuGroundDetection for NewGroundDef {
    fn new_ground(&self) -> bool {
        self.new_ground
    }
    fn lgciu_12_inv(&self) -> bool {
        self.lgciu_12_inv
    }
}

trait Ground {
    fn ground(&self) -> bool;
    fn ground_immediate(&self) -> bool;
}

struct GroundDetection {
    memory1: MemoryNode,
    memory2: MemoryNode,
    conf1: ConfirmationNode,
    mrtrig1: MonostableTriggerNode,
    ground_immediate: bool,
    ground: bool,
}

impl GroundDetection {
    fn new() -> Self {
        Self {
            memory1: MemoryNode::new(true),
            memory2: MemoryNode::new(true),
            conf1: ConfirmationNode::new(true, Duration::from_secs(1)),
            mrtrig1: MonostableTriggerNode::new_retriggerable(true, Duration::from_secs(10)),
            ground_immediate: false,
            ground: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        signals: &(impl EssLhLgCompressed + NormLhLgCompressed + RadioHeight),
        lgciu_ground: &impl LgciuGroundDetection,
    ) {
        let radio_1_ncd = signals.radio_height(1).is_ncd();
        let radio_1_inv = signals.radio_height(1).is_inv();
        let radio_2_ncd = signals.radio_height(2).is_ncd();
        let radio_2_inv = signals.radio_height(2).is_inv();

        let reset_memory =
            !signals.ess_lh_lg_compressed().as_bool() || !signals.norm_lh_lg_compressed().as_bool();
        let set_memory_1 = signals.radio_height(1).as_f64() < 5.0;
        let memory1_out = self.memory1.update(context, set_memory_1, reset_memory);

        let set_memory_2 = signals.radio_height(2).as_f64() < 5.0;
        let memory2_out = self.memory2.update(context, set_memory_2, reset_memory);

        let radio_1_on_gnd = (memory1_out || set_memory_1) && !radio_1_ncd && !radio_1_inv;
        let radio_2_on_gnd = (memory2_out || set_memory_2) && !radio_2_ncd && !radio_2_inv;

        let ground_signals = [
            signals.ess_lh_lg_compressed().as_bool(),
            signals.norm_lh_lg_compressed().as_bool(),
            radio_1_on_gnd,
            radio_2_on_gnd,
        ];
        let ground_count = ground_signals.iter().filter(|&n| *n).count();
        let more_than_2 = ground_count > 2;
        let more_than_1 = ground_count > 1;

        let dual_radio_inv = radio_1_inv && radio_2_inv;
        let gnd_cond1 = more_than_2 && !dual_radio_inv;
        let gnd_cond2 = more_than_1 && dual_radio_inv;

        let mrtrig_in = radio_1_ncd && radio_2_ncd && !lgciu_ground.lgciu_12_inv();
        let trig_ground = self.mrtrig1.update(context, mrtrig_in) && lgciu_ground.new_ground();

        self.ground_immediate = (gnd_cond1 || gnd_cond2) || trig_ground;
        self.ground = self.conf1.update(context, self.ground_immediate);
    }
}

impl Ground for GroundDetection {
    fn ground(&self) -> bool {
        self.ground
    }
    fn ground_immediate(&self) -> bool {
        self.ground_immediate
    }
}

trait AcSpeedAbove80Kt {
    fn ac_speed_above_80_kt(&self) -> bool;

    fn adc_test_inhib(&self) -> bool;
}

struct SpeedDetection {
    conf1: ConfirmationNode,
    conf2: ConfirmationNode,
    conf3: ConfirmationNode,
    memory: MemoryNode,
    mtrig1: MonostableTriggerNode,
    mtrig2: MonostableTriggerNode,
    ac_speed_above_80_kt: bool,
    adc_test_inhib: bool,
}

impl SpeedDetection {
    fn new() -> Self {
        Self {
            conf1: ConfirmationNode::new(true, Duration::from_secs(1)),
            conf2: ConfirmationNode::new(true, Duration::from_secs(1)),
            conf3: ConfirmationNode::new(true, Duration::from_secs(1)),
            memory: MemoryNode::new(true),
            mtrig1: MonostableTriggerNode::new(false, Duration::from_secs_f64(0.5)),
            mtrig2: MonostableTriggerNode::new(false, Duration::from_secs_f64(1.5)),
            ac_speed_above_80_kt: false,
            adc_test_inhib: false,
        }
    }

    fn update(&mut self, context: &UpdateContext, signals: &impl ComputedSpeed) {
        let adc_1_invalid =
            signals.computed_speed(1).is_inv() || signals.computed_speed(1).is_ncd();
        let adc_2_invalid =
            signals.computed_speed(2).is_inv() || signals.computed_speed(2).is_ncd();
        let adc_3_invalid =
            signals.computed_speed(3).is_inv() || signals.computed_speed(3).is_ncd();
        let any_adc_invalid = adc_1_invalid || adc_2_invalid || adc_3_invalid;

        let conf1_out = self.conf1.update(
            context,
            signals.computed_speed(1).as_f64() > 50.0 && !adc_1_invalid,
        );
        let conf2_out = self.conf2.update(
            context,
            signals.computed_speed(2).as_f64() > 50.0 && !adc_2_invalid,
        );
        let conf3_out = self.conf3.update(
            context,
            signals.computed_speed(3).as_f64() > 50.0 && !adc_3_invalid,
        );

        let adc_1_above_80_kt =
            conf1_out && !adc_1_invalid && signals.computed_speed(1).as_f64() > 83.0;
        let adc_2_above_80_kt =
            conf2_out && !adc_2_invalid && signals.computed_speed(2).as_f64() > 83.0;
        let adc_3_above_80_kt =
            conf3_out && !adc_3_invalid && signals.computed_speed(3).as_f64() > 83.0;
        let any_adc_above_80_kt = adc_1_above_80_kt || adc_2_above_80_kt || adc_3_above_80_kt;

        let set_signals = &[
            adc_1_above_80_kt,
            adc_2_above_80_kt,
            adc_3_above_80_kt,
            any_adc_above_80_kt && any_adc_invalid,
        ];
        let set_memory = set_signals.iter().filter(|&n| *n).count() > 1;

        let adc_1_below_77_kt = signals.computed_speed(1).as_f64() < 77.0 && !adc_1_invalid;
        let adc_2_below_77_kt = signals.computed_speed(2).as_f64() < 77.0 && !adc_2_invalid;
        let adc_3_below_77_kt = signals.computed_speed(3).as_f64() < 77.0 && !adc_3_invalid;
        let any_adc_below_77_kt = adc_1_below_77_kt || adc_2_below_77_kt || adc_3_below_77_kt;

        let any_adc_fault = signals.computed_speed(1).is_ft()
            || signals.computed_speed(2).is_ft()
            || signals.computed_speed(3).is_ft();

        let reset_signals = &[
            adc_1_below_77_kt,
            adc_2_below_77_kt,
            adc_3_below_77_kt,
            any_adc_below_77_kt && any_adc_invalid,
        ];
        let reset_memory = reset_signals.iter().filter(|&n| *n).count() > 1
            || self.mtrig1.update(context, any_adc_fault);

        self.ac_speed_above_80_kt = self.memory.update(context, set_memory, reset_memory);
        self.adc_test_inhib = self.mtrig2.update(context, any_adc_fault);
    }
}

impl AcSpeedAbove80Kt for SpeedDetection {
    fn ac_speed_above_80_kt(&self) -> bool {
        self.ac_speed_above_80_kt
    }
    fn adc_test_inhib(&self) -> bool {
        self.adc_test_inhib
    }
}

trait EngineNotRunning {
    fn eng_1_not_running(&self) -> bool;
    fn eng_2_not_running(&self) -> bool;
}

struct EnginesNotRunning {
    trans1: TransientDetectionNode,
    conf1: ConfirmationNode,
    conf2: ConfirmationNode,
    conf3: ConfirmationNode,
    conf4: ConfirmationNode,
    conf5: ConfirmationNode,
    eng_1_not_running: bool,
    eng_2_not_running: bool,
}

impl EnginesNotRunning {
    fn new() -> Self {
        Self {
            trans1: TransientDetectionNode::new(true),
            conf1: ConfirmationNode::new_leading(Duration::from_secs(30)),
            conf2: ConfirmationNode::new_leading(Duration::from_secs(30)),
            conf3: ConfirmationNode::new_leading(Duration::from_secs(30)),
            conf4: ConfirmationNode::new_leading(Duration::from_secs(30)),
            conf5: ConfirmationNode::new_falling(Duration::from_secs(31)),
            eng_1_not_running: false,
            eng_2_not_running: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        signals: &(impl Eng1MasterLevelSelectOn
              + Eng1CoreSpeedAtOrAboveIdle
              + Eng1FirePbOut
              + Eng2CoreSpeedAtOrAboveIdle
              + Eng2MasterLevelSelectOn),
        ground: &impl Ground,
    ) {
        let eng1_core_speed_at_or_above_idle_a = signals.eng1_core_speed_at_or_above_idle(1);
        let eng1_core_speed_at_or_above_idle_b = signals.eng1_core_speed_at_or_above_idle(2);

        let conf5_out = self
            .trans1
            .update(context, signals.eng_1_fire_pb_out().as_bool());

        let conf1_out = self
            .conf1
            .update(context, eng1_core_speed_at_or_above_idle_a.as_bool());

        let conf2_out = self
            .conf2
            .update(context, eng1_core_speed_at_or_above_idle_b.as_bool());

        let eng_1_core_speed_not_running_conf = !conf1_out && !conf2_out;
        let eng_1_core_speed_running_immediate = eng1_core_speed_at_or_above_idle_a.as_bool()
            && eng1_core_speed_at_or_above_idle_b.as_bool()
            && conf5_out
            && !ground.ground();

        let eng_1_core_speed_not_running =
            eng_1_core_speed_not_running_conf && !eng_1_core_speed_running_immediate;

        self.eng_1_not_running = (signals.eng1_master_lever_select_on().is_val()
            && !signals.eng1_master_lever_select_on().as_bool())
            || eng_1_core_speed_not_running;

        let eng2_core_speed_at_or_above_idle_a = signals.eng2_core_speed_at_or_above_idle(1);
        let eng2_core_speed_at_or_above_idle_b = signals.eng2_core_speed_at_or_above_idle(2);

        let conf3_out = self
            .conf3
            .update(context, eng2_core_speed_at_or_above_idle_a.as_bool());

        let conf4_out = self
            .conf4
            .update(context, eng2_core_speed_at_or_above_idle_b.as_bool());

        let eng_2_core_speed_running_immediate = !ground.ground()
            && conf5_out
            && eng1_core_speed_at_or_above_idle_b.as_bool()
            && eng1_core_speed_at_or_above_idle_a.as_bool();

        let eng_2_core_speed_not_running_conf = !conf3_out && !conf4_out;

        let eng_2_core_speed_not_running =
            !eng_2_core_speed_running_immediate && eng_2_core_speed_not_running_conf;

        self.eng_2_not_running = eng_2_core_speed_not_running
            || (!signals.eng1_master_lever_select_on().as_bool()
                && signals.eng2_master_lever_select_on().is_val());
    }
}

impl EngineNotRunning for EnginesNotRunning {
    fn eng_1_not_running(&self) -> bool {
        self.eng_1_not_running
    }

    fn eng_2_not_running(&self) -> bool {
        self.eng_2_not_running
    }
}

trait EngRunning {
    fn eng_1_and_2_not_running(&self) -> bool;
    fn eng_1_or_2_running(&self) -> bool;
    fn one_eng_running(&self) -> bool;
}

struct BothEngineRunning {
    conf1: ConfirmationNode,
    eng_1_and_2_not_running: bool,
    eng_1_or_2_running: bool,
    one_eng_running: bool,
}

impl BothEngineRunning {
    fn new() -> Self {
        Self {
            conf1: ConfirmationNode::new(true, Duration::from_secs(30)),
            eng_1_and_2_not_running: false,
            eng_1_or_2_running: false,
            one_eng_running: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        signals: &(impl EngineNotRunning + Eng1CoreSpeedAtOrAboveIdle + Eng2CoreSpeedAtOrAboveIdle),
    ) {
        self.eng_1_and_2_not_running = signals.eng_1_not_running() && signals.eng_2_not_running();
        let one_eng_running = signals.eng1_core_speed_at_or_above_idle(1).as_bool()
            || signals.eng1_core_speed_at_or_above_idle(2).as_bool()
            || signals.eng2_core_speed_at_or_above_idle(1).as_bool()
            || signals.eng2_core_speed_at_or_above_idle(2).as_bool();

        self.one_eng_running = one_eng_running;
        self.eng_1_or_2_running = self.conf1.update(context, one_eng_running);
    }
}

impl EngRunning for BothEngineRunning {
    fn eng_1_and_2_not_running(&self) -> bool {
        self.eng_1_and_2_not_running
    }

    fn eng_1_or_2_running(&self) -> bool {
        self.eng_1_or_2_running
    }

    fn one_eng_running(&self) -> bool {
        self.one_eng_running
    }
}

trait TakeoffPower {
    fn cfm_flex(&self) -> bool;
    fn eng_1_or_2_to_pwr(&self) -> bool;
}

struct CfmFlightPhases {
    conf1: ConfirmationNode,
    cfm_flex: bool,
    eng_1_or_2_to_pwr: bool,
}

impl CfmFlightPhases {
    fn new() -> Self {
        Self {
            conf1: ConfirmationNode::new(true, Duration::from_secs(30)),
            cfm_flex: false,
            eng_1_or_2_to_pwr: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        //signals: &(impl EngineNotRunning + Eng1CoreSpeedAtOrAboveIdle + Eng2CoreSpeedAtOrAboveIdle),
    ) {
        // todo
    }
}

impl TakeoffPower for CfmFlightPhases {
    fn cfm_flex(&self) -> bool {
        self.cfm_flex
    }

    fn eng_1_or_2_to_pwr(&self) -> bool {
        self.eng_1_or_2_to_pwr
    }
}

trait FlightPhases1 {
    fn phase_1(&self) -> bool;
    fn phase_2(&self) -> bool;
    fn phase_3(&self) -> bool;
    fn phase_4(&self) -> bool;
    fn phase_8(&self) -> bool;
    fn phase_9(&self) -> bool;
    fn phase_10(&self) -> bool;
}

struct FlightPhasesGround {
    trans1: TransientDetectionNode,
    conf1: ConfirmationNode,
    mtrig1: MonostableTriggerNode,
    mtrig2: MonostableTriggerNode,
    mtrig3: MonostableTriggerNode,
    mtrig4: MonostableTriggerNode,
    mtrig5: MonostableTriggerNode,
    mtrig6: MonostableTriggerNode,
    mem_phase10: MemoryNode,
    mem_phase9: MemoryNode,
    prec_phase9: PreceedingValueNode,
    phase1: bool,
    phase2: bool,
    phase3: bool,
    phase4: bool,
    phase8: bool,
    phase9: bool,
    phase10: bool,
}

impl FlightPhasesGround {
    fn new() -> Self {
        Self {
            trans1: TransientDetectionNode::new(false),
            conf1: ConfirmationNode::new_leading(Duration::from_secs_f64(0.2)),
            mtrig1: MonostableTriggerNode::new(false, Duration::from_secs_f64(1.0)),
            mtrig2: MonostableTriggerNode::new(false, Duration::from_secs_f64(3.0)),
            mtrig3: MonostableTriggerNode::new(true, Duration::from_secs_f64(300.0)),
            mtrig4: MonostableTriggerNode::new(true, Duration::from_secs_f64(2.0)),
            mtrig5: MonostableTriggerNode::new(true, Duration::from_secs_f64(2.0)),
            mtrig6: MonostableTriggerNode::new(true, Duration::from_secs_f64(2.0)),
            mem_phase9: MemoryNode::new_nvm(true),
            mem_phase10: MemoryNode::new(false),
            prec_phase9: PreceedingValueNode::new(),
            phase1: false,
            phase2: false,
            phase3: false,
            phase4: false,
            phase8: false,
            phase9: false,
            phase10: false,
        }
    }

    fn update(
        &mut self,
        context: &UpdateContext,
        signals: &(impl Eng1FirePbOut + ToConfigTest),
        ground_sheet: impl Ground,
        ac_speed_sheet: impl AcSpeedAbove80Kt,
        eng_running_sheet: impl EngRunning,
        takeoff_power_sheet: impl TakeoffPower,
    ) {
        let ground = ground_sheet.ground();
        let ground_immediate = ground_sheet.ground_immediate();
        let ac_speed_above_80_kt = ac_speed_sheet.ac_speed_above_80_kt();
        let eng1_or_2_running = eng_running_sheet.eng_1_or_2_running();
        let eng1_or_2_to_pwr = takeoff_power_sheet.eng_1_or_2_to_pwr();

        // phase 1 and 10 preamble
        let trans1 = self
            .trans1
            .update(context, signals.eng_1_fire_pb_out().as_bool());
        let conf1 = self.conf1.update(context, trans1);
        let mtrig5 = self.mtrig5.update(context, conf1);
        let reset_mem10 = ground && mtrig5;

        // phases 3 and 4

        let ground_and_to_pwr = ground && eng1_or_2_to_pwr;

        let phase3 = !ac_speed_above_80_kt && eng1_or_2_running && ground_and_to_pwr;
        self.phase3 = phase3;
        self.phase4 = ac_speed_above_80_kt && ground_and_to_pwr;

        // phase 8

        let phase8_cond1 = ground_immediate || self.mtrig6.update(context, ground_immediate);

        let phase8 = phase8_cond1 & !eng1_or_2_to_pwr && ac_speed_above_80_kt;
        self.phase8 = phase8;

        // phases 2 and 9

        let prec_phase9 = self.prec_phase9.get();
        let mtrig1 = self.mtrig1.update(context, eng1_or_2_to_pwr);
        let mtrig2 = self.mtrig2.update(context, prec_phase9);
        let mtrig4 = self.mtrig4.update(context, !ac_speed_above_80_kt);
        let phase29_cond = ground && !eng1_or_2_to_pwr && !ac_speed_above_80_kt;
        let one_eng_running = eng_running_sheet.one_eng_running();

        let reset_nvm_cond1 = ground && mtrig2;
        let reset_nvm_cond2 = reset_mem10;
        let reset_nvm_cond3 = ground && mtrig1;
        let reset_nvm = reset_nvm_cond1 || reset_nvm_cond2 || reset_nvm_cond3;

        let inhibited_reset_nvm = !mtrig4 && reset_nvm && !prec_phase9;

        let adc_test_inhib = ac_speed_sheet.adc_test_inhib();
        let to_config_test = signals.to_config_test().as_bool();
        let to_config_reset_9 = to_config_test && phase29_cond && one_eng_running;
        let reset_mem9 = inhibited_reset_nvm || adc_test_inhib || to_config_reset_9;

        let phase9_mem = self
            .mem_phase9
            .update(context, phase3 || phase8, reset_mem9);

        self.phase2 = phase29_cond && !phase9_mem && eng1_or_2_running;

        let phase9 = one_eng_running && phase9_mem && phase29_cond;
        self.phase9 = phase9;
        self.prec_phase9.update(context, phase9);

        // phases 1 and 10

        let set_mem10 = phase9;
        let mem_phase10_out = self.mem_phase10.update(context, set_mem10, reset_mem10);

        let phase110_cond =
            !set_mem10 && eng_running_sheet.eng_1_and_2_not_running() && ground_immediate;
        let mtrig3 = self
            .mtrig3
            .update(context, mem_phase10_out && phase110_cond);

        self.phase1 = phase110_cond && !mtrig3;
        self.phase10 = phase110_cond && mtrig3;
    }
}

impl FlightPhases1 for FlightPhasesGround {
    fn phase_1(&self) -> bool {
        self.phase1
    }

    fn phase_2(&self) -> bool {
        self.phase2
    }

    fn phase_3(&self) -> bool {
        self.phase3
    }

    fn phase_4(&self) -> bool {
        self.phase4
    }

    fn phase_8(&self) -> bool {
        self.phase8
    }

    fn phase_9(&self) -> bool {
        self.phase9
    }

    fn phase_10(&self) -> bool {
        self.phase10
    }
}

struct A320FlightWarningComputer {
    new_ground_def: NewGroundDef,
    general_flight_phases: GroundDetection,
    general_flight_phases_above_80: SpeedDetection,
}

impl A320FlightWarningComputer {
    fn new() -> Self {
        Self {
            new_ground_def: NewGroundDef::new(),
            general_flight_phases: GroundDetection::new(),
            general_flight_phases_above_80: SpeedDetection::new(),
        }
    }

    fn update(&mut self, context: &UpdateContext) {
        /*let signals = SignalTable::from();
        self.new_ground_def.update(context, &signals);
        self.general_flight_phases
            .update(context, &signals, &self.new_ground_def);
        self.general_flight_phases_above_80
            .update(context, &signals);*/
    }
}

#[cfg(test)]
mod tests {
    use uom::si::f64::*;
    use uom::si::{
        acceleration::foot_per_second_squared, length::foot,
        thermodynamic_temperature::degree_celsius, velocity::knot,
    };

    use super::*;

    #[cfg(test)]
    mod new_ground_def_tests {
        use super::*;

        #[test]
        fn when_all_compressed_new_ground_and_not_inv() {
            let mut sheet = NewGroundDef::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with()
                    .lh_lg_compressed(1)
                    .ess_lh_lg_compressed()
                    .lh_lg_compressed(2)
                    .norm_lh_lg_compressed()
                    .signals(),
            );
            assert_eq!(sheet.new_ground, true);
            assert_eq!(sheet.lgciu_12_inv, false);
        }

        #[test]
        fn when_none_compressed_new_ground_and_not_inv() {
            let mut sheet = NewGroundDef::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with()
                    .lh_lg_extended(1)
                    .lh_lg_extended(2)
                    .signals(),
            );
            assert_eq!(sheet.new_ground, false);
            assert_eq!(sheet.lgciu_12_inv, false);
        }

        #[test]
        fn when_single_lgciu_mismatch_then_lgciu12_inv() {
            let mut sheet = NewGroundDef::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with()
                    .lh_lg_compressed(1)
                    .lh_lg_extended(2)
                    .signals(),
            );
            assert_eq!(sheet.new_ground, false);
            assert_eq!(sheet.lgciu_12_inv, true);
        }
    }

    #[cfg(test)]
    mod general_flight_phases_definition {
        use super::*;

        struct TestLgciuGroundDetection {
            new_ground: bool,
            lgciu_12_inv: bool,
        }

        impl TestLgciuGroundDetection {
            fn new(new_ground: bool, lgciu_12_inv: bool) -> Self {
                Self {
                    new_ground: new_ground,
                    lgciu_12_inv: lgciu_12_inv,
                }
            }
        }

        impl LgciuGroundDetection for TestLgciuGroundDetection {
            fn new_ground(&self) -> bool {
                self.new_ground
            }
            fn lgciu_12_inv(&self) -> bool {
                self.lgciu_12_inv
            }
        }

        #[test]
        fn when_on_ground_ground_immediate_and_ground() {
            let mut sheet = GroundDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with()
                    .ess_lh_lg_compressed()
                    .norm_lh_lg_compressed()
                    .radio_heights(0.0, -1.0)
                    .signals(),
                &TestLgciuGroundDetection::new(true, false),
            );
            assert_eq!(sheet.ground_immediate, true);
            assert_eq!(sheet.ground, true);
        }

        #[test]
        fn when_touching_down_triggers_ground_immediate_first() {
            let mut sheet = GroundDetection::new();
            sheet.update(
                &gnd_context(Duration::from_millis(500)),
                test_bed_with()
                    .ess_lh_lg_compressed()
                    .norm_lh_lg_compressed()
                    .radio_heights(0.0, -1.0)
                    .signals(),
                &TestLgciuGroundDetection::new(true, false),
            );
            assert_eq!(sheet.ground_immediate, true);
            assert_eq!(sheet.ground, false);
            sheet.update(
                &gnd_context(Duration::from_millis(500)),
                test_bed_with()
                    .ess_lh_lg_compressed()
                    .norm_lh_lg_compressed()
                    .radio_heights(0.0, -1.0)
                    .signals(),
                &TestLgciuGroundDetection::new(true, false),
            );
            assert_eq!(sheet.ground_immediate, true);
            assert_eq!(sheet.ground, true);
        }
    }

    #[cfg(test)]
    mod general_flight_phases_definition_above_80_kt {
        use super::*;

        #[test]
        fn when_at_0_kt_not_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(0.0, 0.0, 0.0).signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), false);
        }

        #[test]
        fn when_at_250_kt_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with()
                    .computed_speeds(250.0, 250.0, 250.0)
                    .signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), true);
        }

        #[test]
        fn when_one_adc_at_250_kt_not_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(250.0, 0.0, 0.0).signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), false);
        }

        #[test]
        fn when_two_at_250_kt_and_adc_failure_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speed_1(250.0).signals(), // todo ADC failures
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), true);
        }

        #[test]
        fn when_two_adcs_at_250_kt_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(250.0, 0.0, 250.0).signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), true);
        }

        #[test]
        fn when_spikes_below_50_to_above_80_kt_not_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(49.0, 49.0, 49.0).signals(),
            );
            sheet.update(
                &gnd_context(Duration::from_secs_f64(0.5)),
                test_bed_with().computed_speeds(84.0, 84.0, 84.0).signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), false);
        }

        #[test]
        fn when_jumps_below_50_to_above_80_kt_above_80_kt() {
            let mut sheet = SpeedDetection::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(49.0, 49.0, 49.0).signals(),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().computed_speeds(84.0, 84.0, 84.0).signals(),
            );
            assert_eq!(sheet.ac_speed_above_80_kt(), true);
        }
    }

    #[cfg(test)]
    mod general_flight_phases_1 {
        use super::*;
        use crate::flight_warning::tests::test_bed_with;

        /*struct TestSignals {
            eng_1_fire_pb_out: DiscreteParameter,
            eng_1_and_2_not_running: bool,
            ground_immediate: bool,
            ac_speed_abv_80kt: bool,
            one_eng_running: bool,
            adc_test_inhib: bool,
            eng_1_or_2_running: bool,
            ground: bool,
            eng_1_or_2_to_pwr: bool,
            to_config_test: Arinc429Parameter<bool>,
        }

        impl TestSignals {
            fn new_cold_and_dark() -> Self {
                Self {
                    eng_1_fire_pb_out: DiscreteParameter::new(false),
                    eng_1_and_2_not_running: true,
                    ground_immediate: true,
                    ac_speed_abv_80kt: false,
                    one_eng_running: false,
                    adc_test_inhib: false,
                    eng_1_or_2_running: false,
                    ground: true,
                    eng_1_or_2_to_pwr: false,
                    to_config_test: Arinc429Parameter::new(false),
                }
            }

            fn new_ground_one_engine_running() -> Self {
                Self {
                    eng_1_fire_pb_out: DiscreteParameter::new(false),
                    eng_1_and_2_not_running: false,
                    ground_immediate: true,
                    ac_speed_abv_80kt: false,
                    one_eng_running: true,
                    adc_test_inhib: false,
                    eng_1_or_2_running: true,
                    ground: true,
                    eng_1_or_2_to_pwr: false,
                    to_config_test: Arinc429Parameter::new(false),
                }
            }

            fn new_ground_takeoff_power() -> Self {
                Self {
                    eng_1_fire_pb_out: DiscreteParameter::new(false),
                    eng_1_and_2_not_running: false,
                    ground_immediate: true,
                    ac_speed_abv_80kt: false,
                    one_eng_running: true,
                    adc_test_inhib: false,
                    eng_1_or_2_running: true,
                    ground: true,
                    eng_1_or_2_to_pwr: true,
                    to_config_test: Arinc429Parameter::new(false),
                }
            }

            fn new_ground_above_80_knots(takeoff_pwr: bool) -> Self {
                Self {
                    eng_1_fire_pb_out: DiscreteParameter::new(false),
                    eng_1_and_2_not_running: false,
                    ground_immediate: true,
                    ac_speed_abv_80kt: true,
                    one_eng_running: true,
                    adc_test_inhib: false,
                    eng_1_or_2_running: true,
                    ground: true,
                    eng_1_or_2_to_pwr: takeoff_pwr,
                    to_config_test: Arinc429Parameter::new(false),
                }
            }

            fn new_ground_rollout(to_config_test: bool) -> Self {
                Self {
                    eng_1_fire_pb_out: DiscreteParameter::new(false),
                    eng_1_and_2_not_running: false,
                    ground_immediate: true,
                    ac_speed_abv_80kt: false,
                    one_eng_running: true,
                    adc_test_inhib: false,
                    eng_1_or_2_running: true,
                    ground: true,
                    eng_1_or_2_to_pwr: false,
                    to_config_test: Arinc429Parameter::new(to_config_test),
                }
            }
        }*/

        trait GetPhase {
            fn get_phase(&self) -> usize;
        }
        impl GetPhase for FlightPhasesGround {
            fn get_phase(&self) -> usize {
                let phase_signals = [
                    self.phase_1(),
                    self.phase_2(),
                    self.phase_3(),
                    self.phase_4(),
                    self.phase_8(),
                    self.phase_9(),
                    self.phase_10(),
                ];
                let phase_count = phase_signals.iter().filter(|&n| *n).count();
                assert_eq!(
                    phase_count, 1,
                    "expected exactly one active phase, got {} active phases",
                    phase_count
                );
                if self.phase_1() {
                    return 1;
                } else if self.phase_2() {
                    return 2;
                } else if self.phase_3() {
                    return 3;
                } else if self.phase_4() {
                    return 4;
                } else if self.phase_8() {
                    return 8;
                } else if self.phase_9() {
                    return 9;
                } else if self.phase_10() {
                    return 10;
                }
                panic!();
            }
        }

        /*#[test]
        fn cold_and_dark_is_phase_1() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().signals(),
            );
            assert_eq!(sheet.get_phase(), 1);
        }

        #[test]
        fn after_engine_start_is_phase_2() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_one_engine_running(),
            );
            assert_eq!(sheet.get_phase(), 2);
        }

        #[test]
        fn on_runway_at_takeoff_power_is_phase_3() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_takeoff_power(),
            );
            assert_eq!(sheet.get_phase(), 3);
        }

        #[test]
        fn on_runway_at_takeoff_power_above_80_knots_is_phase_4() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(true),
            );
            assert_eq!(sheet.get_phase(), 4);
        }

        #[test]
        fn on_runway_at_idle_above_80_knots_is_phase_8() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(false),
            );
            assert_eq!(sheet.get_phase(), 8);
        }

        #[test]
        fn after_high_speed_rto_below_80_knots_is_phase_9() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(true),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(false),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_rollout(false),
            );
            assert_eq!(sheet.get_phase(), 9);
        }

        #[test]
        fn after_high_speed_rto_below_80_knots_and_to_config_is_phase_2() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(true),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_above_80_knots(false),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_rollout(false),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                test_bed_with().takeoff_config_test_pressed().signals(),
            );
            assert_eq!(sheet.get_phase(), 2);
        }

        #[test]
        fn after_engine_shutdown_reset_to_phase_1() {
            let mut sheet = FlightPhasesGround::new();
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_ground_takeoff_power(),
            );
            assert_eq!(sheet.get_phase(), 3);
            sheet.update(
                &gnd_context(Duration::from_secs(30)),
                &TestSignals::new_ground_above_80_knots(true),
            );
            sheet.update(
                &gnd_context(Duration::from_secs(60)),
                &TestSignals::new_ground_one_engine_running(),
            );
            assert_eq!(sheet.get_phase(), 9);
            sheet.update(
                &gnd_context(Duration::from_secs(60)),
                &TestSignals::new_cold_and_dark(),
            );
            assert_eq!(sheet.get_phase(), 10);
            sheet.update(
                &gnd_context(Duration::from_secs(300)),
                &TestSignals::new_cold_and_dark(),
            );
            assert_eq!(sheet.get_phase(), 1);
            sheet.update(
                &gnd_context(Duration::from_secs(1)),
                &TestSignals::new_cold_and_dark(),
            );
        }*/
    }

    fn gnd_context(delta_time: Duration) -> UpdateContext {
        UpdateContext::new(
            delta_time,
            Velocity::new::<knot>(0.),
            Length::new::<foot>(0.),
            ThermodynamicTemperature::new::<degree_celsius>(25.0),
            true,
            Acceleration::new::<foot_per_second_squared>(0.),
        )
    }

    struct A320SignalTable {
        lh_lg_compressed_1: Arinc429Parameter<bool>,
        lh_lg_compressed_2: Arinc429Parameter<bool>,
        ess_lh_lg_compressed: DiscreteParameter,
        norm_lh_lg_compressed: DiscreteParameter,
        radio_height_1: Arinc429Parameter<f64>,
        radio_height_2: Arinc429Parameter<f64>,
        computed_speed_1: Arinc429Parameter<f64>,
        computed_speed_2: Arinc429Parameter<f64>,
        computed_speed_3: Arinc429Parameter<f64>,
        eng1_master_lever_select_on: Arinc429Parameter<bool>,
        eng2_master_lever_select_on: Arinc429Parameter<bool>,
        eng1_core_speed_at_or_above_idle_a: Arinc429Parameter<bool>,
        eng1_core_speed_at_or_above_idle_b: Arinc429Parameter<bool>,
        eng2_core_speed_at_or_above_idle_a: Arinc429Parameter<bool>,
        eng2_core_speed_at_or_above_idle_b: Arinc429Parameter<bool>,
        eng_1_fire_pb_out: DiscreteParameter,
        to_config_test: Arinc429Parameter<bool>,
    }
    impl A320SignalTable {
        fn new() -> Self {
            Self {
                lh_lg_compressed_1: Arinc429Parameter::new_inv(false),
                lh_lg_compressed_2: Arinc429Parameter::new_inv(false),
                ess_lh_lg_compressed: DiscreteParameter::new_inv(false),
                norm_lh_lg_compressed: DiscreteParameter::new_inv(false),
                radio_height_1: Arinc429Parameter::new_inv(0.0),
                radio_height_2: Arinc429Parameter::new_inv(0.0),
                computed_speed_1: Arinc429Parameter::new_inv(0.0),
                computed_speed_2: Arinc429Parameter::new_inv(0.0),
                computed_speed_3: Arinc429Parameter::new_inv(0.0),
                eng1_master_lever_select_on: Arinc429Parameter::new_inv(false),
                eng2_master_lever_select_on: Arinc429Parameter::new_inv(false),
                eng1_core_speed_at_or_above_idle_a: Arinc429Parameter::new_inv(false),
                eng1_core_speed_at_or_above_idle_b: Arinc429Parameter::new_inv(false),
                eng2_core_speed_at_or_above_idle_a: Arinc429Parameter::new_inv(false),
                eng2_core_speed_at_or_above_idle_b: Arinc429Parameter::new_inv(false),
                eng_1_fire_pb_out: DiscreteParameter::new_inv(false),
                to_config_test: Arinc429Parameter::new_inv(false),
            }
        }

        fn set_takeoff_config_test(&mut self, pressed: bool) {
            self.to_config_test = Arinc429Parameter::new(pressed)
        }

        fn set_computed_speed_1(&mut self, speed: Arinc429Parameter<f64>) {
            self.computed_speed_1 = speed;
        }

        fn set_computed_speed_2(&mut self, speed: Arinc429Parameter<f64>) {
            self.computed_speed_2 = speed;
        }

        fn set_computed_speed_3(&mut self, speed: Arinc429Parameter<f64>) {
            self.computed_speed_3 = speed;
        }

        fn set_lh_lg_compressed_1(&mut self, compressed: Arinc429Parameter<bool>) {
            self.lh_lg_compressed_1 = compressed;
        }

        fn set_lh_lg_compressed_2(&mut self, compressed: Arinc429Parameter<bool>) {
            self.lh_lg_compressed_2 = compressed;
        }

        fn set_ess_lh_lg_compressed(&mut self, compressed: DiscreteParameter) {
            self.ess_lh_lg_compressed = compressed;
        }

        fn set_norm_lh_lg_compressed(&mut self, compressed: DiscreteParameter) {
            self.norm_lh_lg_compressed = compressed;
        }

        fn set_radio_height_1(&mut self, height: Arinc429Parameter<f64>) {
            self.radio_height_1 = height;
        }

        fn set_radio_height_2(&mut self, height: Arinc429Parameter<f64>) {
            self.radio_height_2 = height;
        }
    }
    impl LhLgCompressed for A320SignalTable {
        fn lh_lg_compressed(&self, index: usize) -> &Arinc429Parameter<bool> {
            match index {
                1 => &self.lh_lg_compressed_1,
                2 => &self.lh_lg_compressed_2,
                _ => panic!(),
            }
        }
    }
    impl EssLhLgCompressed for A320SignalTable {
        fn ess_lh_lg_compressed(&self) -> &DiscreteParameter {
            &self.ess_lh_lg_compressed
        }
    }
    impl NormLhLgCompressed for A320SignalTable {
        fn norm_lh_lg_compressed(&self) -> &DiscreteParameter {
            &self.norm_lh_lg_compressed
        }
    }
    impl RadioHeight for A320SignalTable {
        fn radio_height(&self, index: usize) -> &Arinc429Parameter<f64> {
            match index {
                1 => &self.radio_height_1,
                2 => &self.radio_height_2,
                _ => panic!(),
            }
        }
    }
    impl ComputedSpeed for A320SignalTable {
        fn computed_speed(&self, index: usize) -> &Arinc429Parameter<f64> {
            match index {
                1 => &self.computed_speed_1,
                2 => &self.computed_speed_2,
                3 => &self.computed_speed_3,
                _ => panic!(),
            }
        }
    }
    impl Eng1MasterLevelSelectOn for A320SignalTable {
        fn eng1_master_lever_select_on(&self) -> &Arinc429Parameter<bool> {
            &self.eng1_master_lever_select_on
        }
    }
    impl Eng2MasterLevelSelectOn for A320SignalTable {
        fn eng2_master_lever_select_on(&self) -> &Arinc429Parameter<bool> {
            &self.eng2_master_lever_select_on
        }
    }
    impl Eng1CoreSpeedAtOrAboveIdle for A320SignalTable {
        fn eng1_core_speed_at_or_above_idle(&self, index: usize) -> &Arinc429Parameter<bool> {
            match index {
                1 => &self.eng1_core_speed_at_or_above_idle_a,
                2 => &self.eng1_core_speed_at_or_above_idle_b,
                _ => panic!(),
            }
        }
    }
    impl Eng2CoreSpeedAtOrAboveIdle for A320SignalTable {
        fn eng2_core_speed_at_or_above_idle(&self, index: usize) -> &Arinc429Parameter<bool> {
            match index {
                1 => &self.eng2_core_speed_at_or_above_idle_a,
                2 => &self.eng2_core_speed_at_or_above_idle_b,
                _ => panic!(),
            }
        }
    }
    impl Eng1FirePbOut for A320SignalTable {
        fn eng_1_fire_pb_out(&self) -> &DiscreteParameter {
            &self.eng_1_fire_pb_out
        }
    }
    impl ToConfigTest for A320SignalTable {
        fn to_config_test(&self) -> &Arinc429Parameter<bool> {
            &self.to_config_test
        }
    }

    struct A320SignalTestBed {
        signals: A320SignalTable,
    }
    impl A320SignalTestBed {
        fn new() -> Self {
            let mut signals = A320SignalTable::new();
            Self { signals }
        }

        fn and(self) -> Self {
            self
        }

        fn signals(&self) -> &A320SignalTable {
            &self.signals
        }

        fn takeoff_config_test_pressed(mut self) -> Self {
            self.signals.set_takeoff_config_test(true);
            self
        }

        fn computed_speeds(mut self, speed1: f64, speed2: f64, speed3: f64) -> Self {
            self.signals
                .set_computed_speed_1(Arinc429Parameter::new(speed1));
            self.signals
                .set_computed_speed_2(Arinc429Parameter::new(speed2));
            self.signals
                .set_computed_speed_3(Arinc429Parameter::new(speed3));
            self
        }

        fn computed_speed_1(mut self, speed: f64) -> Self {
            self.signals
                .set_computed_speed_1(Arinc429Parameter::new(speed));
            self
        }

        fn computed_speed_2(mut self, speed: f64) -> Self {
            self.signals
                .set_computed_speed_2(Arinc429Parameter::new(speed));
            self
        }

        fn computed_speed_3(mut self, speed: f64) -> Self {
            self.signals
                .set_computed_speed_3(Arinc429Parameter::new(speed));
            self
        }

        fn lh_lg_compressed(mut self, lgciu: usize) -> Self {
            match lgciu {
                1 => self
                    .signals
                    .set_lh_lg_compressed_1(Arinc429Parameter::new(true)),
                2 => self
                    .signals
                    .set_lh_lg_compressed_2(Arinc429Parameter::new(true)),
                _ => panic!(),
            }
            self
        }

        fn lh_lg_extended(mut self, lgciu: usize) -> Self {
            match lgciu {
                1 => self
                    .signals
                    .set_lh_lg_compressed_1(Arinc429Parameter::new(false)),
                2 => self
                    .signals
                    .set_lh_lg_compressed_2(Arinc429Parameter::new(false)),
                _ => panic!(),
            }
            self
        }

        fn ess_lh_lg_compressed(mut self) -> Self {
            self.signals
                .set_ess_lh_lg_compressed(DiscreteParameter::new(true));
            self
        }

        fn norm_lh_lg_compressed(mut self) -> Self {
            self.signals
                .set_norm_lh_lg_compressed(DiscreteParameter::new(true));
            self
        }

        fn radio_heights(mut self, height1: f64, height2: f64) -> Self {
            self.signals
                .set_radio_height_1(Arinc429Parameter::new(height1));
            self.signals
                .set_radio_height_2(Arinc429Parameter::new(height2));
            self
        }
    }

    fn test_bed() -> A320SignalTestBed {
        A320SignalTestBed::new()
    }

    fn test_bed_with() -> A320SignalTestBed {
        test_bed()
    }
}