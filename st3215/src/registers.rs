/** Auto-generated code. Do not modify. See build.rs for details. */

pub enum RegisterAddress {
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
    FirmwareMajorVersionNumber = 0,

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
    FirmwareSubVersionNumber = 1,

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
    ServoMainVersionNumber = 3,

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
    ServoSubVersionNumber = 4,

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
    ID = 5,

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
    BaudRate = 6,

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
    ReturnDelay = 7,

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
    ResponseStatusLevel = 8,

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
    MinimumAngleLimitation = 9,

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
    MaximumAngleLimitation = 11,

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
    MaximumTemperatureLimit = 13,

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
    MaximumInputVoltage = 14,

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
    MinimumInputVoltage = 15,

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
    MaximumTorque = 16,

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
    Phase = 18,

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
    UnloadingCondition = 19,

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
    LEDAlarmCondition = 20,

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
    ProportionalityCoefficient = 21,

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
    DifferentialCoefficient = 22,

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
    IntegralCoefficient = 23,

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
    MinimumStartupForce = 24,

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
    ClockwiseInsensitiveArea = 26,

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
    CounterclockwiseInsensitiveRegion = 27,

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
    ProtectionCurrent = 28,

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
    AngularResolution = 30,

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
    PositionCorrection = 31,

    /**
     * Operation mode
     *
     * 0: position servo mode
     * 1: The motor is in constant speed mode, which is controlled by parameter 0x2e, and the highest bit 15 is the direction bit
     * 2: PWM open-loop speed regulation mode, with parameter 0x2c running time parameter control, bit11 as direction bit
     * 3: In step servo mode, the number of step progress is represented by parameter 0x2a, and the highest bit 15 is the direction bit“
     *
     * initial_value: 0
     * storage_area: EPROM
     * authority: read&write
     * minimum_value: 0
     * maximum_value: 2
     * unit: no
     */
    OperationMode = 33,

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
    ProtectiveTorque = 34,

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
    ProtectionTime = 35,

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
    OverloadTorque = 36,

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
    SpeedClosedLoopPProportionalCoefficient = 37,

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
    OverCurrentProtectionTime = 38,

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
    VelocityClosedLoopIIntegralCoefficient = 39,

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
    TorqueSwitch = 40,

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
    Acceleration = 41,

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
    TargetLocation = 42,

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
    RunningTime = 44,

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
    RunningSpeed = 46,

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
    TorqueLimit = 48,

    /**
     * Lock mark
     *
     * Write 0 closes the write lock, and the value written to EPROM address is saved after power failure.
     * Write 1 opens the write lock, and the value written to EPROM address is not saved after power failure
     *
     * initial_value: 0
     * storage_area: SRAM
     * authority: read&write
     * minimum_value: 0
     * maximum_value: 1
     * unit: no
     */
    LockMark = 55,

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
    CurrentLocation = 56,

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
    CurrentSpeed = 58,

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
    CurrentLoad = 60,

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
    CurrentVoltage = 62,

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
    CurrentTemperature = 63,

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
    AsynchronousWriteFlag = 64,

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
    ServoStatus = 65,

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
    MobileSign = 66,

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
    CurrentCurrent = 69,
}

impl RegisterAddress {
    pub fn from_memory_address(memory_address: u8) -> Option<Self> {
        match memory_address {
            0 => Some(Self::FirmwareMajorVersionNumber),
            1 => Some(Self::FirmwareSubVersionNumber),
            3 => Some(Self::ServoMainVersionNumber),
            4 => Some(Self::ServoSubVersionNumber),
            5 => Some(Self::ID),
            6 => Some(Self::BaudRate),
            7 => Some(Self::ReturnDelay),
            8 => Some(Self::ResponseStatusLevel),
            9 => Some(Self::MinimumAngleLimitation),
            11 => Some(Self::MaximumAngleLimitation),
            13 => Some(Self::MaximumTemperatureLimit),
            14 => Some(Self::MaximumInputVoltage),
            15 => Some(Self::MinimumInputVoltage),
            16 => Some(Self::MaximumTorque),
            18 => Some(Self::Phase),
            19 => Some(Self::UnloadingCondition),
            20 => Some(Self::LEDAlarmCondition),
            21 => Some(Self::ProportionalityCoefficient),
            22 => Some(Self::DifferentialCoefficient),
            23 => Some(Self::IntegralCoefficient),
            24 => Some(Self::MinimumStartupForce),
            26 => Some(Self::ClockwiseInsensitiveArea),
            27 => Some(Self::CounterclockwiseInsensitiveRegion),
            28 => Some(Self::ProtectionCurrent),
            30 => Some(Self::AngularResolution),
            31 => Some(Self::PositionCorrection),
            33 => Some(Self::OperationMode),
            34 => Some(Self::ProtectiveTorque),
            35 => Some(Self::ProtectionTime),
            36 => Some(Self::OverloadTorque),
            37 => Some(Self::SpeedClosedLoopPProportionalCoefficient),
            38 => Some(Self::OverCurrentProtectionTime),
            39 => Some(Self::VelocityClosedLoopIIntegralCoefficient),
            40 => Some(Self::TorqueSwitch),
            41 => Some(Self::Acceleration),
            42 => Some(Self::TargetLocation),
            44 => Some(Self::RunningTime),
            46 => Some(Self::RunningSpeed),
            48 => Some(Self::TorqueLimit),
            55 => Some(Self::LockMark),
            56 => Some(Self::CurrentLocation),
            58 => Some(Self::CurrentSpeed),
            60 => Some(Self::CurrentLoad),
            62 => Some(Self::CurrentVoltage),
            63 => Some(Self::CurrentTemperature),
            64 => Some(Self::AsynchronousWriteFlag),
            65 => Some(Self::ServoStatus),
            66 => Some(Self::MobileSign),
            69 => Some(Self::CurrentCurrent),
            _ => None,
        }
    }

    pub fn length(&self) -> u8 {
        match self {
            Self::FirmwareMajorVersionNumber => 1,
            Self::FirmwareSubVersionNumber => 1,
            Self::ServoMainVersionNumber => 1,
            Self::ServoSubVersionNumber => 1,
            Self::ID => 1,
            Self::BaudRate => 1,
            Self::ReturnDelay => 1,
            Self::ResponseStatusLevel => 1,
            Self::MinimumAngleLimitation => 2,
            Self::MaximumAngleLimitation => 2,
            Self::MaximumTemperatureLimit => 1,
            Self::MaximumInputVoltage => 1,
            Self::MinimumInputVoltage => 1,
            Self::MaximumTorque => 2,
            Self::Phase => 1,
            Self::UnloadingCondition => 1,
            Self::LEDAlarmCondition => 1,
            Self::ProportionalityCoefficient => 1,
            Self::DifferentialCoefficient => 1,
            Self::IntegralCoefficient => 1,
            Self::MinimumStartupForce => 2,
            Self::ClockwiseInsensitiveArea => 1,
            Self::CounterclockwiseInsensitiveRegion => 1,
            Self::ProtectionCurrent => 2,
            Self::AngularResolution => 1,
            Self::PositionCorrection => 2,
            Self::OperationMode => 1,
            Self::ProtectiveTorque => 1,
            Self::ProtectionTime => 1,
            Self::OverloadTorque => 1,
            Self::SpeedClosedLoopPProportionalCoefficient => 1,
            Self::OverCurrentProtectionTime => 1,
            Self::VelocityClosedLoopIIntegralCoefficient => 1,
            Self::TorqueSwitch => 1,
            Self::Acceleration => 1,
            Self::TargetLocation => 2,
            Self::RunningTime => 2,
            Self::RunningSpeed => 2,
            Self::TorqueLimit => 2,
            Self::LockMark => 1,
            Self::CurrentLocation => 2,
            Self::CurrentSpeed => 2,
            Self::CurrentLoad => 2,
            Self::CurrentVoltage => 1,
            Self::CurrentTemperature => 1,
            Self::AsynchronousWriteFlag => 1,
            Self::ServoStatus => 1,
            Self::MobileSign => 1,
            Self::CurrentCurrent => 2,
        }
    }
}