
/*
RegisterDescription {
    memory_address: "0x00",
    function: "Firmware major version number",
    bytes: "1",
    initial_value: "3",
    storage_area: "EPROM",
    authority: "read",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "",
    analysis_of_values: "",
}
*/
/*
RegisterDescription {
    memory_address: "0x01",
    function: "Firmware sub version number",
    bytes: "1",
    initial_value: "6",
    storage_area: "EPROM",
    authority: "read",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "",
    analysis_of_values: "",
}
*/
/*
RegisterDescription {
    memory_address: "0x03",
    function: "servo Main Version Number",
    bytes: "1",
    initial_value: "9",
    storage_area: "EPROM",
    authority: "read",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "",
    analysis_of_values: "",
}
*/
/*
RegisterDescription {
    memory_address: "0x04",
    function: "servo sub version number",
    bytes: "1",
    initial_value: "3",
    storage_area: "EPROM",
    authority: "read",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "",
    analysis_of_values: "",
}
*/
/*
RegisterDescription {
    memory_address: "0x05",
    function: "ID",
    bytes: "1",
    initial_value: "1",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "253",
    unit: "number",
    analysis_of_values: "Unique identification code on the bus. Duplicate ID number is not allowed on the same bus,254 (oxfe) is the broadcast ID, broadcast does not return a reply packet“",
}
*/
/*
RegisterDescription {
    memory_address: "0x06",
    function: "Baud rate",
    bytes: "1",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "7",
    unit: "no",
    analysis_of_values: "0-7 represents baud rate as follows: 1000000 500000 250000 128000 115200 76800 57600 38400",
}
*/
/*
RegisterDescription {
    memory_address: "0x07",
    function: "Return delay",
    bytes: "1",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "2us",
    analysis_of_values: "The minimum unit is 2us, and the maximum set return delay is 254 * 2 = 508us",
}
*/
/*
RegisterDescription {
    memory_address: "0x08",
    function: "Response status level",
    bytes: "1",
    initial_value: "1",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1",
    unit: "no",
    analysis_of_values: "0: except for read instruction and Ping instruction, other instructions do not return reply packet;1: Returns a reply packet for all instructions“",
}
*/
/*
RegisterDescription {
    memory_address: "0x09",
    function: "Minimum Angle Limitation",
    bytes: "2",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "4094",
    unit: "step",
    analysis_of_values: "Set the minimum limit of motion stroke, the value is less than the maximum angle limit, and this value is 0 when the multi cycle absolute position control is carried out",
}
*/
/*
RegisterDescription {
    memory_address: "0x11",
    function: "Maximum Angle Limitation",
    bytes: "2",
    initial_value: "4095",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "1",
    maximum_value: "4095",
    unit: "step",
    analysis_of_values: "Set the maximum limit of motion stroke, which is greater than the minimum angle limit, and the value is 0 when the multi turn absolute position control is adopted.",
}
*/
/*
RegisterDescription {
    memory_address: "0x13",
    function: "Maximum Temperature Limit",
    bytes: "1",
    initial_value: "70",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "100",
    unit: "°C",
    analysis_of_values: "The maximum operating temperature limit, if set to 70, the maximum temperature is 70 ℃, and the setting accuracy is 1 ℃",
}
*/
/*
RegisterDescription {
    memory_address: "0x14",
    function: "Maximum input voltage",
    bytes: "1",
    initial_value: "80",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "0.1V",
    analysis_of_values: "If the maximum input voltage is set to 80, the maximum working voltage is limited to 8.0V and the setting accuracy is 0.1V",
}
*/
/*
RegisterDescription {
    memory_address: "0x15",
    function: "Minimum input voltage",
    bytes: "1",
    initial_value: "40",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "0.1V",
    analysis_of_values: "If the minimum input voltage is set to 40, the minimum working voltage is limited to 4.0V and the setting accuracy is 0.1V",
}
*/
/*
RegisterDescription {
    memory_address: "0x16",
    function: "Maximum torque",
    bytes: "2",
    initial_value: "1000",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1000",
    unit: "0.10%",
    analysis_of_values: "Set the maximum output torque limit of the servo, and set 1000 = 100% * locked torque,Power on assigned to address 48 torque limit“",
}
*/
/*
RegisterDescription {
    memory_address: "0x18",
    function: "phase",
    bytes: "1",
    initial_value: "12",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "Special function byte, which cannot be modified without special requirements. See special byte bit analysis for details",
}
*/
/*
RegisterDescription {
    memory_address: "0x19",
    function: "Unloading condition",
    bytes: "1",
    initial_value: "44",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable corresponding protection",
}
*/
/*
RegisterDescription {
    memory_address: "0x20",
    function: "LED Alarm condition",
    bytes: "1",
    initial_value: "47",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to close the corresponding protection“Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable flashing alarm. The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to turn off flashing light alarm“",
}
*/
/*
RegisterDescription {
    memory_address: "0x21",
    function: "P Proportionality coefficient",
    bytes: "1",
    initial_value: "32",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "Proportional factor of control motor",
}
*/
/*
RegisterDescription {
    memory_address: "0x22",
    function: "D Differential coefficient",
    bytes: "1",
    initial_value: "32",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "Differential coefficient of control motor",
}
*/
/*
RegisterDescription {
    memory_address: "0x23",
    function: "I Integral coefficient",
    bytes: "1",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "Integral coefficient of control motor",
}
*/
/*
RegisterDescription {
    memory_address: "0x24",
    function: "Minimum startup force",
    bytes: "2",
    initial_value: "16",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1000",
    unit: "0.1%",
    analysis_of_values: "Set the minimum output starting torque of servo and set 1000 = 100% * locked torque",
}
*/
/*
RegisterDescription {
    memory_address: "0x26",
    function: "Clockwise insensitive area",
    bytes: "1",
    initial_value: "1",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "32",
    unit: "step",
    analysis_of_values: "The minimum unit is a minimum resolution angle",
}
*/
/*
RegisterDescription {
    memory_address: "0x27",
    function: "Counterclockwise insensitive region",
    bytes: "1",
    initial_value: "1",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "32",
    unit: "step",
    analysis_of_values: "The minimum unit is a minimum resolution angle",
}
*/
/*
RegisterDescription {
    memory_address: "0x28",
    function: "Protection current",
    bytes: "2",
    initial_value: "500",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "511",
    unit: "6.5mA",
    analysis_of_values: "The maximum current can be set at 3255ma",
}
*/
/*
RegisterDescription {
    memory_address: "0x30",
    function: "Angular resolution",
    bytes: "1",
    initial_value: "1",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "1",
    maximum_value: "100",
    unit: "no",
    analysis_of_values: "For the amplification factor of minimum resolution angle (degree / step), the number of control turns can be extended by modifying this value",
}
*/
/*
RegisterDescription {
    memory_address: "0x31",
    function: "Position correction",
    bytes: "2",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "-2047",
    maximum_value: "2047",
    unit: "step",
    analysis_of_values: "Bit11 is the direction bit, indicating the positive and negative directions. Other bits can represent the range of 0-2047 steps",
}
*/
/*
RegisterDescription {
    memory_address: "0x33",
    function: "Operation mode",
    bytes: "1",
    initial_value: "0",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "2",
    unit: "no",
    analysis_of_values: "0: position servo mode\\n1: The motor is in constant speed mode, which is controlled by parameter 0x2e, and the highest bit 15 is the direction bit\\n2: PWM open-loop speed regulation mode, with parameter 0x2c running time parameter control, bit11 as direction bit\\n3: In step servo mode, the number of step progress is represented by parameter 0x2a, and the highest bit 15 is the direction bit“",
}
*/
/*
RegisterDescription {
    memory_address: "0x34",
    function: "Protective torque",
    bytes: "1",
    initial_value: "20",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "1.0%",
    analysis_of_values: "After entering the overload protection, the output torque, if set to 20, means 20% of the maximum torque",
}
*/
/*
RegisterDescription {
    memory_address: "0x35",
    function: "Protection time",
    bytes: "1",
    initial_value: "200",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "10ms",
    analysis_of_values: "The timing time when the current load output exceeds the overload torque and remains. If 200 is set to 2 seconds, the maximum can be set to 2.5 seconds",
}
*/
/*
RegisterDescription {
    memory_address: "0x36",
    function: "Overload torque",
    bytes: "1",
    initial_value: "80",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "1.0%",
    analysis_of_values: "The maximum torque threshold of starting overload protection time meter, if set to 80, means 80% of the maximum torque",
}
*/
/*
RegisterDescription {
    memory_address: "0x37",
    function: "Speed closed loop P proportional coefficient",
    bytes: "1",
    initial_value: "10",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "In the motor constant speed mode (mode 1), the speed loop proportional coefficient",
}
*/
/*
RegisterDescription {
    memory_address: "0x38",
    function: "Over current protection time",
    bytes: "1",
    initial_value: "200",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "10ms",
    analysis_of_values: "The maximum setting is 254 * 10ms = 2540ms",
}
*/
/*
RegisterDescription {
    memory_address: "0x39",
    function: "Velocity closed loop I integral coefficient",
    bytes: "1",
    initial_value: "10",
    storage_area: "EPROM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "no",
    analysis_of_values: "In the motor constant speed mode (mode 1), the speed loop integral coefficient",
}
*/
/*
RegisterDescription {
    memory_address: "0x28",
    function: "Torque switch",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "2",
    unit: "no",
    analysis_of_values: "Write 0: turn off torque output; write 1: turn on torque output; write 128: current position correction is 2048",
}
*/
/*
RegisterDescription {
    memory_address: "0x41",
    function: "acceleration",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "100step/s^2",
    analysis_of_values: "If it is set to 10, the speed will be changed by 1000 steps per second",
}
*/
/*
RegisterDescription {
    memory_address: "0x42",
    function: "Target location",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "-32766",
    maximum_value: "32766",
    unit: "step",
    analysis_of_values: "Each step is a minimum resolution angle, absolute position control mode, the maximum corresponding to the maximum effective angle",
}
*/
/*
RegisterDescription {
    memory_address: "0x44",
    function: "Running time",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1000",
    unit: "0.10%",
    analysis_of_values: "",
}
*/
/*
RegisterDescription {
    memory_address: "0x46",
    function: "running speed",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "254",
    unit: "step/s",
    analysis_of_values: "Number of steps in unit time (per second), 50 steps / second = 0.732 RPM (cycles per minute)",
}
*/
/*
RegisterDescription {
    memory_address: "0x48",
    function: "Torque limit",
    bytes: "2",
    initial_value: "1000",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1000",
    unit: "1.0%",
    analysis_of_values: "The initial value of power on is assigned by the maximum torque (0x10), which can be modified by the user to control the output of the maximum torque",
}
*/
/*
RegisterDescription {
    memory_address: "0x55",
    function: "Lock mark",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read&write",
    minimum_value: "0",
    maximum_value: "1",
    unit: "no",
    analysis_of_values: "Write 0 closes the write lock, and the value written to EPROM address is saved after power failure.\\nWrite 1 opens the write lock, and the value written to EPROM address is not saved after power failure",
}
*/
/*
RegisterDescription {
    memory_address: "0x56",
    function: "current location",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "step",
    analysis_of_values: "In the absolute position control mode, the maximum value corresponds to the maximum effective angle",
}
*/
/*
RegisterDescription {
    memory_address: "0x58",
    function: "Current speed",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "step/s",
    analysis_of_values: "Feedback the current speed of motor rotation, the number of steps in unit time (per second)",
}
*/
/*
RegisterDescription {
    memory_address: "0x60",
    function: "Current load",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "0.1%",
    analysis_of_values: "Voltage duty cycle of current control output drive motor",
}
*/
/*
RegisterDescription {
    memory_address: "0x62",
    function: "Current voltage",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "0.1V",
    analysis_of_values: "Current servo working voltage",
}
*/
/*
RegisterDescription {
    memory_address: "0x63",
    function: "Current temperature",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "°C",
    analysis_of_values: "Current internal operating temperature of the servo",
}
*/
/*
RegisterDescription {
    memory_address: "0x64",
    function: "Asynchronous write flag",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "no",
    analysis_of_values: "When using asynchronous write instruction, flag bit",
}
*/
/*
RegisterDescription {
    memory_address: "0x65",
    function: "Servo status",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "no",
    analysis_of_values: "Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to 1, indicating that the corresponding error occurs,Voltage sensor temperature current angle overload corresponding bit 0 is no phase error.",
}
*/
/*
RegisterDescription {
    memory_address: "0x66",
    function: "Mobile sign",
    bytes: "1",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "no",
    analysis_of_values: "When the servo is moving, it is marked as 1, and when the servo is stopped, it is 0",
}
*/
/*
RegisterDescription {
    memory_address: "0x69",
    function: "Current current",
    bytes: "2",
    initial_value: "0",
    storage_area: "SRAM",
    authority: "read-only",
    minimum_value: "-1",
    maximum_value: "-1",
    unit: "6.5mA",
    analysis_of_values: "The maximum measurable current is 500 * 6.5ma = 3250ma",
}
*/
