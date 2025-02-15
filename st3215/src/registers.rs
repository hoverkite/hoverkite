/** Auto-generated code. Do not modify. */

trait Register {
    type Value;
    
    const MEMORY_ADDRESS: u8;
}


/**
 * Firmware major version number
 *
 * initial_value: 3
 * storage_area: EPROM
 * authority: read
 * minimum_value: -1
 * maximum_value: -1
 * unit:
 */
pub struct FirmwareMajorVersionNumber;
impl Register for FirmwareMajorVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0;
}

/**
 * Firmware sub version number
 *
 * initial_value: 6
 * storage_area: EPROM
 * authority: read
 * minimum_value: -1
 * maximum_value: -1
 * unit:
 */
pub struct FirmwareSubVersionNumber;
impl Register for FirmwareSubVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 1;
}

/**
 * servo Main Version Number
 *
 * initial_value: 9
 * storage_area: EPROM
 * authority: read
 * minimum_value: -1
 * maximum_value: -1
 * unit:
 */
pub struct ServoMainVersionNumber;
impl Register for ServoMainVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 3;
}

/**
 * servo sub version number
 *
 * initial_value: 3
 * storage_area: EPROM
 * authority: read
 * minimum_value: -1
 * maximum_value: -1
 * unit:
 */
pub struct ServoSubVersionNumber;
impl Register for ServoSubVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 4;
}

/**
 * ID
 *
 * Unique identification code on the bus. Duplicate ID number is not allowed on the same bus,254 (oxfe) is the broadcast ID, broadcast does not return a reply packet“
 *
 * initial_value: 1
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 253
 * unit: number
 */
pub struct ID;
impl Register for ID {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 5;
}

/**
 * Baud rate
 *
 * 0-7 represents baud rate as follows: 1000000 500000 250000 128000 115200 76800 57600 38400
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 7
 * unit: no
 */
pub struct BaudRate;
impl Register for BaudRate {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 6;
}

/**
 * Return delay
 *
 * The minimum unit is 2us, and the maximum set return delay is 254 * 2 = 508us
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 2us
 */
pub struct ReturnDelay;
impl Register for ReturnDelay {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 7;
}

/**
 * Response status level
 *
 * 0: except for read instruction and Ping instruction, other instructions do not return reply packet;1: Returns a reply packet for all instructions“
 *
 * initial_value: 1
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1
 * unit: no
 */
pub struct ResponseStatusLevel;
impl Register for ResponseStatusLevel {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 8;
}

/**
 * Minimum Angle Limitation
 *
 * Set the minimum limit of motion stroke, the value is less than the maximum angle limit, and this value is 0 when the multi cycle absolute position control is carried out
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 4094
 * unit: step
 */
pub struct MinimumAngleLimitation;
impl Register for MinimumAngleLimitation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 9;
}

/**
 * Maximum Angle Limitation
 *
 * Set the maximum limit of motion stroke, which is greater than the minimum angle limit, and the value is 0 when the multi turn absolute position control is adopted.
 *
 * initial_value: 4095
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 1
 * maximum_value: 4095
 * unit: step
 */
pub struct MaximumAngleLimitation;
impl Register for MaximumAngleLimitation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 11;
}

/**
 * Maximum Temperature Limit
 *
 * The maximum operating temperature limit, if set to 70, the maximum temperature is 70 ℃, and the setting accuracy is 1 ℃
 *
 * initial_value: 70
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 100
 * unit: °C
 */
pub struct MaximumTemperatureLimit;
impl Register for MaximumTemperatureLimit {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 13;
}

/**
 * Maximum input voltage
 *
 * If the maximum input voltage is set to 80, the maximum working voltage is limited to 8.0V and the setting accuracy is 0.1V
 *
 * initial_value: 80
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 0.1V
 */
pub struct MaximumInputVoltage;
impl Register for MaximumInputVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 14;
}

/**
 * Minimum input voltage
 *
 * If the minimum input voltage is set to 40, the minimum working voltage is limited to 4.0V and the setting accuracy is 0.1V
 *
 * initial_value: 40
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 0.1V
 */
pub struct MinimumInputVoltage;
impl Register for MinimumInputVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 15;
}

/**
 * Maximum torque
 *
 * Set the maximum output torque limit of the servo, and set 1000 = 100% * locked torque,Power on assigned to address 48 torque limit“
 *
 * initial_value: 1000
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1000
 * unit: 0.10%
 */
pub struct MaximumTorque;
impl Register for MaximumTorque {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 16;
}

/**
 * phase
 *
 * Special function byte, which cannot be modified without special requirements. See special byte bit analysis for details
 *
 * initial_value: 12
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct Phase;
impl Register for Phase {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 18;
}

/**
 * Unloading condition
 *
 * Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable corresponding protection
 *
 * initial_value: 44
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct UnloadingCondition;
impl Register for UnloadingCondition {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 19;
}

/**
 * LED Alarm condition
 *
 * The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to close the corresponding protection“Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable flashing alarm. The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to turn off flashing light alarm“
 *
 * initial_value: 47
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct LEDAlarmCondition;
impl Register for LEDAlarmCondition {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 20;
}

/**
 * P Proportionality coefficient
 *
 * Proportional factor of control motor
 *
 * initial_value: 32
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct PProportionalityCoefficient;
impl Register for PProportionalityCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 21;
}

/**
 * D Differential coefficient
 *
 * Differential coefficient of control motor
 *
 * initial_value: 32
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct DDifferentialCoefficient;
impl Register for DDifferentialCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 22;
}

/**
 * I Integral coefficient
 *
 * Integral coefficient of control motor
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct IIntegralCoefficient;
impl Register for IIntegralCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 23;
}

/**
 * Minimum startup force
 *
 * Set the minimum output starting torque of servo and set 1000 = 100% * locked torque
 *
 * initial_value: 16
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1000
 * unit: 0.1%
 */
pub struct MinimumStartupForce;
impl Register for MinimumStartupForce {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 24;
}

/**
 * Clockwise insensitive area
 *
 * The minimum unit is a minimum resolution angle
 *
 * initial_value: 1
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 32
 * unit: step
 */
pub struct ClockwiseInsensitiveArea;
impl Register for ClockwiseInsensitiveArea {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 26;
}

/**
 * Counterclockwise insensitive region
 *
 * The minimum unit is a minimum resolution angle
 *
 * initial_value: 1
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 32
 * unit: step
 */
pub struct CounterclockwiseInsensitiveRegion;
impl Register for CounterclockwiseInsensitiveRegion {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 27;
}

/**
 * Protection current
 *
 * The maximum current can be set at 3255ma
 *
 * initial_value: 500
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 511
 * unit: 6.5mA
 */
pub struct ProtectionCurrent;
impl Register for ProtectionCurrent {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 28;
}

/**
 * Angular resolution
 *
 * For the amplification factor of minimum resolution angle (degree / step), the number of control turns can be extended by modifying this value
 *
 * initial_value: 1
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 1
 * maximum_value: 100
 * unit: no
 */
pub struct AngularResolution;
impl Register for AngularResolution {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 30;
}

/**
 * Position correction
 *
 * Bit11 is the direction bit, indicating the positive and negative directions. Other bits can represent the range of 0-2047 steps
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: -2047
 * maximum_value: 2047
 * unit: step
 */
pub struct PositionCorrection;
impl Register for PositionCorrection {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 31;
}

/**
 * Operation mode
 *
 * 0: position servo mode\n1: The motor is in constant speed mode, which is controlled by parameter 0x2e, and the highest bit 15 is the direction bit\n2: PWM open-loop speed regulation mode, with parameter 0x2c running time parameter control, bit11 as direction bit\n3: In step servo mode, the number of step progress is represented by parameter 0x2a, and the highest bit 15 is the direction bit“
 *
 * initial_value: 0
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 2
 * unit: no
 */
pub struct OperationMode;
impl Register for OperationMode {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 33;
}

/**
 * Protective torque
 *
 * After entering the overload protection, the output torque, if set to 20, means 20% of the maximum torque
 *
 * initial_value: 20
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 1.0%
 */
pub struct ProtectiveTorque;
impl Register for ProtectiveTorque {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 34;
}

/**
 * Protection time
 *
 * The timing time when the current load output exceeds the overload torque and remains. If 200 is set to 2 seconds, the maximum can be set to 2.5 seconds
 *
 * initial_value: 200
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 10ms
 */
pub struct ProtectionTime;
impl Register for ProtectionTime {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 35;
}

/**
 * Overload torque
 *
 * The maximum torque threshold of starting overload protection time meter, if set to 80, means 80% of the maximum torque
 *
 * initial_value: 80
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 1.0%
 */
pub struct OverloadTorque;
impl Register for OverloadTorque {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 36;
}

/**
 * Speed closed loop P proportional coefficient
 *
 * In the motor constant speed mode (mode 1), the speed loop proportional coefficient
 *
 * initial_value: 10
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct SpeedClosedLoopPProportionalCoefficient;
impl Register for SpeedClosedLoopPProportionalCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 37;
}

/**
 * Over current protection time
 *
 * The maximum setting is 254 * 10ms = 2540ms
 *
 * initial_value: 200
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 10ms
 */
pub struct OverCurrentProtectionTime;
impl Register for OverCurrentProtectionTime {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 38;
}

/**
 * Velocity closed loop I integral coefficient
 *
 * In the motor constant speed mode (mode 1), the speed loop integral coefficient
 *
 * initial_value: 10
 * storage_area: EPROM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: no
 */
pub struct VelocityClosedLoopIIntegralCoefficient;
impl Register for VelocityClosedLoopIIntegralCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 39;
}

/**
 * Torque switch
 *
 * Write 0: turn off torque output; write 1: turn on torque output; write 128: current position correction is 2048
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 2
 * unit: no
 */
pub struct TorqueSwitch;
impl Register for TorqueSwitch {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 40;
}

/**
 * acceleration
 *
 * If it is set to 10, the speed will be changed by 1000 steps per second
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: 100step/s^2
 */
pub struct Acceleration;
impl Register for Acceleration {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 41;
}

/**
 * Target location
 *
 * Each step is a minimum resolution angle, absolute position control mode, the maximum corresponding to the maximum effective angle
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: -32766
 * maximum_value: 32766
 * unit: step
 */
pub struct TargetLocation;
impl Register for TargetLocation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 42;
}

/**
 * Running time
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1000
 * unit: 0.10%
 */
pub struct RunningTime;
impl Register for RunningTime {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 44;
}

/**
 * running speed
 *
 * Number of steps in unit time (per second), 50 steps / second = 0.732 RPM (cycles per minute)
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 254
 * unit: step/s
 */
pub struct RunningSpeed;
impl Register for RunningSpeed {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 46;
}

/**
 * Torque limit
 *
 * The initial value of power on is assigned by the maximum torque (0x10), which can be modified by the user to control the output of the maximum torque
 *
 * initial_value: 1000
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1000
 * unit: 1.0%
 */
pub struct TorqueLimit;
impl Register for TorqueLimit {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 48;
}

/**
 * Lock mark
 *
 * Write 0 closes the write lock, and the value written to EPROM address is saved after power failure.\nWrite 1 opens the write lock, and the value written to EPROM address is not saved after power failure
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read&write
 * minimum_value: 0
 * maximum_value: 1
 * unit: no
 */
pub struct LockMark;
impl Register for LockMark {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 55;
}

/**
 * current location
 *
 * In the absolute position control mode, the maximum value corresponds to the maximum effective angle
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: step
 */
pub struct CurrentLocation;
impl Register for CurrentLocation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 56;
}

/**
 * Current speed
 *
 * Feedback the current speed of motor rotation, the number of steps in unit time (per second)
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: step/s
 */
pub struct CurrentSpeed;
impl Register for CurrentSpeed {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 58;
}

/**
 * Current load
 *
 * Voltage duty cycle of current control output drive motor
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: 0.1%
 */
pub struct CurrentLoad;
impl Register for CurrentLoad {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 60;
}

/**
 * Current voltage
 *
 * Current servo working voltage
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: 0.1V
 */
pub struct CurrentVoltage;
impl Register for CurrentVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 62;
}

/**
 * Current temperature
 *
 * Current internal operating temperature of the servo
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: °C
 */
pub struct CurrentTemperature;
impl Register for CurrentTemperature {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 63;
}

/**
 * Asynchronous write flag
 *
 * When using asynchronous write instruction, flag bit
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: no
 */
pub struct AsynchronousWriteFlag;
impl Register for AsynchronousWriteFlag {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 64;
}

/**
 * Servo status
 *
 * Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to 1, indicating that the corresponding error occurs,Voltage sensor temperature current angle overload corresponding bit 0 is no phase error.
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: no
 */
pub struct ServoStatus;
impl Register for ServoStatus {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 65;
}

/**
 * Mobile sign
 *
 * When the servo is moving, it is marked as 1, and when the servo is stopped, it is 0
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: no
 */
pub struct MobileSign;
impl Register for MobileSign {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 66;
}

/**
 * Current current
 *
 * The maximum measurable current is 500 * 6.5ma = 3250ma
 *
 * initial_value: 0
 * storage_area: SRAM
 * authority: read-only
 * minimum_value: -1
 * maximum_value: -1
 * unit: 6.5mA
 */
pub struct CurrentCurrent;
impl Register for CurrentCurrent {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 69;
}
pub enum RegisterAddress {
    FirmwareMajorVersionNumber(FirmwareMajorVersionNumber),
    FirmwareSubVersionNumber(FirmwareSubVersionNumber),
    ServoMainVersionNumber(ServoMainVersionNumber),
    ServoSubVersionNumber(ServoSubVersionNumber),
    ID(ID),
    BaudRate(BaudRate),
    ReturnDelay(ReturnDelay),
    ResponseStatusLevel(ResponseStatusLevel),
    MinimumAngleLimitation(MinimumAngleLimitation),
    MaximumAngleLimitation(MaximumAngleLimitation),
    MaximumTemperatureLimit(MaximumTemperatureLimit),
    MaximumInputVoltage(MaximumInputVoltage),
    MinimumInputVoltage(MinimumInputVoltage),
    MaximumTorque(MaximumTorque),
    Phase(Phase),
    UnloadingCondition(UnloadingCondition),
    LEDAlarmCondition(LEDAlarmCondition),
    PProportionalityCoefficient(PProportionalityCoefficient),
    DDifferentialCoefficient(DDifferentialCoefficient),
    IIntegralCoefficient(IIntegralCoefficient),
    MinimumStartupForce(MinimumStartupForce),
    ClockwiseInsensitiveArea(ClockwiseInsensitiveArea),
    CounterclockwiseInsensitiveRegion(CounterclockwiseInsensitiveRegion),
    ProtectionCurrent(ProtectionCurrent),
    AngularResolution(AngularResolution),
    PositionCorrection(PositionCorrection),
    OperationMode(OperationMode),
    ProtectiveTorque(ProtectiveTorque),
    ProtectionTime(ProtectionTime),
    OverloadTorque(OverloadTorque),
    SpeedClosedLoopPProportionalCoefficient(SpeedClosedLoopPProportionalCoefficient),
    OverCurrentProtectionTime(OverCurrentProtectionTime),
    VelocityClosedLoopIIntegralCoefficient(VelocityClosedLoopIIntegralCoefficient),
    TorqueSwitch(TorqueSwitch),
    Acceleration(Acceleration),
    TargetLocation(TargetLocation),
    RunningTime(RunningTime),
    RunningSpeed(RunningSpeed),
    TorqueLimit(TorqueLimit),
    LockMark(LockMark),
    CurrentLocation(CurrentLocation),
    CurrentSpeed(CurrentSpeed),
    CurrentLoad(CurrentLoad),
    CurrentVoltage(CurrentVoltage),
    CurrentTemperature(CurrentTemperature),
    AsynchronousWriteFlag(AsynchronousWriteFlag),
    ServoStatus(ServoStatus),
    MobileSign(MobileSign),
    CurrentCurrent(CurrentCurrent),
}

impl RegisterAddress {
    pub fn from_memory_address(memory_address: u8) -> Option<Self> {
        match memory_address {
            0 => Some(Self::FirmwareMajorVersionNumber(FirmwareMajorVersionNumber)),
            1 => Some(Self::FirmwareSubVersionNumber(FirmwareSubVersionNumber)),
            3 => Some(Self::ServoMainVersionNumber(ServoMainVersionNumber)),
            4 => Some(Self::ServoSubVersionNumber(ServoSubVersionNumber)),
            5 => Some(Self::ID(ID)),
            6 => Some(Self::BaudRate(BaudRate)),
            7 => Some(Self::ReturnDelay(ReturnDelay)),
            8 => Some(Self::ResponseStatusLevel(ResponseStatusLevel)),
            9 => Some(Self::MinimumAngleLimitation(MinimumAngleLimitation)),
            11 => Some(Self::MaximumAngleLimitation(MaximumAngleLimitation)),
            13 => Some(Self::MaximumTemperatureLimit(MaximumTemperatureLimit)),
            14 => Some(Self::MaximumInputVoltage(MaximumInputVoltage)),
            15 => Some(Self::MinimumInputVoltage(MinimumInputVoltage)),
            16 => Some(Self::MaximumTorque(MaximumTorque)),
            18 => Some(Self::Phase(Phase)),
            19 => Some(Self::UnloadingCondition(UnloadingCondition)),
            20 => Some(Self::LEDAlarmCondition(LEDAlarmCondition)),
            21 => Some(Self::PProportionalityCoefficient(PProportionalityCoefficient)),
            22 => Some(Self::DDifferentialCoefficient(DDifferentialCoefficient)),
            23 => Some(Self::IIntegralCoefficient(IIntegralCoefficient)),
            24 => Some(Self::MinimumStartupForce(MinimumStartupForce)),
            26 => Some(Self::ClockwiseInsensitiveArea(ClockwiseInsensitiveArea)),
            27 => Some(Self::CounterclockwiseInsensitiveRegion(CounterclockwiseInsensitiveRegion)),
            28 => Some(Self::ProtectionCurrent(ProtectionCurrent)),
            30 => Some(Self::AngularResolution(AngularResolution)),
            31 => Some(Self::PositionCorrection(PositionCorrection)),
            33 => Some(Self::OperationMode(OperationMode)),
            34 => Some(Self::ProtectiveTorque(ProtectiveTorque)),
            35 => Some(Self::ProtectionTime(ProtectionTime)),
            36 => Some(Self::OverloadTorque(OverloadTorque)),
            37 => Some(Self::SpeedClosedLoopPProportionalCoefficient(SpeedClosedLoopPProportionalCoefficient)),
            38 => Some(Self::OverCurrentProtectionTime(OverCurrentProtectionTime)),
            39 => Some(Self::VelocityClosedLoopIIntegralCoefficient(VelocityClosedLoopIIntegralCoefficient)),
            40 => Some(Self::TorqueSwitch(TorqueSwitch)),
            41 => Some(Self::Acceleration(Acceleration)),
            42 => Some(Self::TargetLocation(TargetLocation)),
            44 => Some(Self::RunningTime(RunningTime)),
            46 => Some(Self::RunningSpeed(RunningSpeed)),
            48 => Some(Self::TorqueLimit(TorqueLimit)),
            55 => Some(Self::LockMark(LockMark)),
            56 => Some(Self::CurrentLocation(CurrentLocation)),
            58 => Some(Self::CurrentSpeed(CurrentSpeed)),
            60 => Some(Self::CurrentLoad(CurrentLoad)),
            62 => Some(Self::CurrentVoltage(CurrentVoltage)),
            63 => Some(Self::CurrentTemperature(CurrentTemperature)),
            64 => Some(Self::AsynchronousWriteFlag(AsynchronousWriteFlag)),
            65 => Some(Self::ServoStatus(ServoStatus)),
            66 => Some(Self::MobileSign(MobileSign)),
            69 => Some(Self::CurrentCurrent(CurrentCurrent)),
            _ => None,
        }
    }

    pub fn length(&self) -> u8 {
        match self {
            Self::FirmwareMajorVersionNumber(_) => 1,
            Self::FirmwareSubVersionNumber(_) => 1,
            Self::ServoMainVersionNumber(_) => 1,
            Self::ServoSubVersionNumber(_) => 1,
            Self::ID(_) => 1,
            Self::BaudRate(_) => 1,
            Self::ReturnDelay(_) => 1,
            Self::ResponseStatusLevel(_) => 1,
            Self::MinimumAngleLimitation(_) => 2,
            Self::MaximumAngleLimitation(_) => 2,
            Self::MaximumTemperatureLimit(_) => 1,
            Self::MaximumInputVoltage(_) => 1,
            Self::MinimumInputVoltage(_) => 1,
            Self::MaximumTorque(_) => 2,
            Self::Phase(_) => 1,
            Self::UnloadingCondition(_) => 1,
            Self::LEDAlarmCondition(_) => 1,
            Self::PProportionalityCoefficient(_) => 1,
            Self::DDifferentialCoefficient(_) => 1,
            Self::IIntegralCoefficient(_) => 1,
            Self::MinimumStartupForce(_) => 2,
            Self::ClockwiseInsensitiveArea(_) => 1,
            Self::CounterclockwiseInsensitiveRegion(_) => 1,
            Self::ProtectionCurrent(_) => 2,
            Self::AngularResolution(_) => 1,
            Self::PositionCorrection(_) => 2,
            Self::OperationMode(_) => 1,
            Self::ProtectiveTorque(_) => 1,
            Self::ProtectionTime(_) => 1,
            Self::OverloadTorque(_) => 1,
            Self::SpeedClosedLoopPProportionalCoefficient(_) => 1,
            Self::OverCurrentProtectionTime(_) => 1,
            Self::VelocityClosedLoopIIntegralCoefficient(_) => 1,
            Self::TorqueSwitch(_) => 1,
            Self::Acceleration(_) => 1,
            Self::TargetLocation(_) => 2,
            Self::RunningTime(_) => 2,
            Self::RunningSpeed(_) => 2,
            Self::TorqueLimit(_) => 2,
            Self::LockMark(_) => 1,
            Self::CurrentLocation(_) => 2,
            Self::CurrentSpeed(_) => 2,
            Self::CurrentLoad(_) => 2,
            Self::CurrentVoltage(_) => 1,
            Self::CurrentTemperature(_) => 1,
            Self::AsynchronousWriteFlag(_) => 1,
            Self::ServoStatus(_) => 1,
            Self::MobileSign(_) => 1,
            Self::CurrentCurrent(_) => 2,
        }
    }
}