/** Auto-generated code. Do not modify. */

trait Register {
    type Value;
    
    const MEMORY_ADDRESS: u8;
    // TODO:
    // const initial_value: Option<Self::Value>;
    // const minimum_value: Option<Self::Value>;
    // const maximum_value: Option<Self::Value>;
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
struct FirmwareMajorVersionNumber;
impl Register for FirmwareMajorVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x00;
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
struct FirmwareSubVersionNumber;
impl Register for FirmwareSubVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x01;
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
struct ServoMainVersionNumber;
impl Register for ServoMainVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x03;
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
struct ServoSubVersionNumber;
impl Register for ServoSubVersionNumber {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x04;
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
struct ID;
impl Register for ID {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x05;
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
struct BaudRate;
impl Register for BaudRate {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x06;
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
struct ReturnDelay;
impl Register for ReturnDelay {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x07;
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
struct ResponseStatusLevel;
impl Register for ResponseStatusLevel {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x08;
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
struct MinimumAngleLimitation;
impl Register for MinimumAngleLimitation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x09;
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
struct MaximumAngleLimitation;
impl Register for MaximumAngleLimitation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x11;
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
struct MaximumTemperatureLimit;
impl Register for MaximumTemperatureLimit {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x13;
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
struct MaximumInputVoltage;
impl Register for MaximumInputVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x14;
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
struct MinimumInputVoltage;
impl Register for MinimumInputVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x15;
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
struct MaximumTorque;
impl Register for MaximumTorque {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x16;
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
struct Phase;
impl Register for Phase {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x18;
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
struct UnloadingCondition;
impl Register for UnloadingCondition {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x19;
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
struct LEDAlarmCondition;
impl Register for LEDAlarmCondition {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x20;
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
struct PProportionalityCoefficient;
impl Register for PProportionalityCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x21;
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
struct DDifferentialCoefficient;
impl Register for DDifferentialCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x22;
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
struct IIntegralCoefficient;
impl Register for IIntegralCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x23;
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
struct MinimumStartupForce;
impl Register for MinimumStartupForce {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x24;
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
struct ClockwiseInsensitiveArea;
impl Register for ClockwiseInsensitiveArea {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x26;
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
struct CounterclockwiseInsensitiveRegion;
impl Register for CounterclockwiseInsensitiveRegion {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x27;
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
struct ProtectionCurrent;
impl Register for ProtectionCurrent {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x28;
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
struct AngularResolution;
impl Register for AngularResolution {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x30;
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
struct PositionCorrection;
impl Register for PositionCorrection {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x31;
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
struct OperationMode;
impl Register for OperationMode {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x33;
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
struct ProtectiveTorque;
impl Register for ProtectiveTorque {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x34;
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
struct ProtectionTime;
impl Register for ProtectionTime {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x35;
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
struct OverloadTorque;
impl Register for OverloadTorque {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x36;
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
struct SpeedClosedLoopPProportionalCoefficient;
impl Register for SpeedClosedLoopPProportionalCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x37;
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
struct OverCurrentProtectionTime;
impl Register for OverCurrentProtectionTime {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x38;
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
struct VelocityClosedLoopIIntegralCoefficient;
impl Register for VelocityClosedLoopIIntegralCoefficient {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x39;
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
struct TorqueSwitch;
impl Register for TorqueSwitch {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x28;
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
struct Acceleration;
impl Register for Acceleration {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x41;
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
struct TargetLocation;
impl Register for TargetLocation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x42;
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
struct RunningTime;
impl Register for RunningTime {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x44;
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
struct RunningSpeed;
impl Register for RunningSpeed {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x46;
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
struct TorqueLimit;
impl Register for TorqueLimit {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x48;
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
struct LockMark;
impl Register for LockMark {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x55;
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
struct CurrentLocation;
impl Register for CurrentLocation {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x56;
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
struct CurrentSpeed;
impl Register for CurrentSpeed {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x58;
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
struct CurrentLoad;
impl Register for CurrentLoad {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x60;
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
struct CurrentVoltage;
impl Register for CurrentVoltage {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x62;
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
struct CurrentTemperature;
impl Register for CurrentTemperature {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x63;
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
struct AsynchronousWriteFlag;
impl Register for AsynchronousWriteFlag {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x64;
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
struct ServoStatus;
impl Register for ServoStatus {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x65;
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
struct MobileSign;
impl Register for MobileSign {
    type Value = u8;
    const MEMORY_ADDRESS: u8 = 0x66;
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
struct CurrentCurrent;
impl Register for CurrentCurrent {
    type Value = u16;
    const MEMORY_ADDRESS: u8 = 0x69;
}
