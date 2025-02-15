/**
 * st3215_memory_table section exported from the spreadsheet as TSV, and simplified so I can parse it without using the CSV crate.
 */
const MEMORY_TABLE_TSV: &str = r#"
Memory address		Function	Bytes	Initial value	Storage area	authority	Minimum value	Maximum value	unit	Analysis of values
DEC HEX		Low Front High behind							If the function address uses two bytes of data, the low byte is in the front address, and the high byte is in the back address
0	0x00	Firmware major version number	1	3	EPROM	read	-1	-1		
1	0x01	Firmware sub version number	1	6	EPROM	read	-1	-1		
3	0x03	servo Main Version Number	1	9	EPROM	read	-1	-1		
4	0x04	servo sub version number	1	3	EPROM	read	-1	-1		
5	0x05	ID	1	1	EPROM	read&write	0	253	number	Unique identification code on the bus. Duplicate ID number is not allowed on the same bus,254 (oxfe) is the broadcast ID, broadcast does not return a reply packet“
6	0x06	Baud rate	1	0	EPROM	read&write	0	7	no	0-7 represents baud rate as follows: 1000000 500000 250000 128000 115200 76800 57600 38400
7	0x07	Return delay	1	0	EPROM	read&write	0	254	2us	The minimum unit is 2us, and the maximum set return delay is 254 * 2 = 508us
8	0x08	Response status level	1	1	EPROM	read&write	0	1	no	0: except for read instruction and Ping instruction, other instructions do not return reply packet;1: Returns a reply packet for all instructions“
9	0x09	Minimum Angle Limitation	2	0	EPROM	read&write	0	4094	step	Set the minimum limit of motion stroke, the value is less than the maximum angle limit, and this value is 0 when the multi cycle absolute position control is carried out
11	0x11	Maximum Angle Limitation	2	4095	EPROM	read&write	1	4095	step	Set the maximum limit of motion stroke, which is greater than the minimum angle limit, and the value is 0 when the multi turn absolute position control is adopted.
13	0x13	Maximum Temperature Limit	1	70	EPROM	read&write	0	100	°C	The maximum operating temperature limit, if set to 70, the maximum temperature is 70 ℃, and the setting accuracy is 1 ℃
14	0x14	Maximum input voltage	1	80	EPROM	read&write	0	254	0.1V	If the maximum input voltage is set to 80, the maximum working voltage is limited to 8.0V and the setting accuracy is 0.1V
15	0x15	Minimum input voltage	1	40	EPROM	read&write	0	254	0.1V	If the minimum input voltage is set to 40, the minimum working voltage is limited to 4.0V and the setting accuracy is 0.1V
16	0x16	Maximum torque	2	1000	EPROM	read&write	0	1000	0.10%	Set the maximum output torque limit of the servo, and set 1000 = 100% * locked torque,Power on assigned to address 48 torque limit“
18	0x18	phase	1	12	EPROM	read&write	0	254	no	Special function byte, which cannot be modified without special requirements. See special byte bit analysis for details
19	0x19	Unloading condition	1	44	EPROM	read&write	0	254	no	Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable corresponding protection
20	0x20	LED Alarm condition	1	47	EPROM	read&write	0	254	no	The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to close the corresponding protection“Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to enable flashing alarm. The corresponding bit of temperature current angle overload of voltage sensor is set to 0 to turn off flashing light alarm“
21	0x21	P Proportionality coefficient	1	32	EPROM	read&write	0	254	no	Proportional factor of control motor
22	0x22	D Differential coefficient	1	32	EPROM	read&write	0	254	no	Differential coefficient of control motor
23	0x23	I Integral coefficient	1	0	EPROM	read&write	0	254	no	Integral coefficient of control motor
24	0x24	Minimum startup force	2	16	EPROM	read&write	0	1000	0.1%	Set the minimum output starting torque of servo and set 1000 = 100% * locked torque
26	0x26	Clockwise insensitive area	1	1	EPROM	read&write	0	32	step	The minimum unit is a minimum resolution angle
27	0x27	Counterclockwise insensitive region	1	1	EPROM	read&write	0	32	step	The minimum unit is a minimum resolution angle
28	0x28	Protection current	2	500	EPROM	read&write	0	511	6.5mA	The maximum current can be set at 3255ma
30	0x30	Angular resolution	1	1	EPROM	read&write	1	100	no	For the amplification factor of minimum resolution angle (degree / step), the number of control turns can be extended by modifying this value
31	0x31	Position correction	2	0	EPROM	read&write	-2047	2047	step	Bit11 is the direction bit, indicating the positive and negative directions. Other bits can represent the range of 0-2047 steps
33	0x33	Operation mode	1	0	EPROM	read&write	0	2	no	0: position servo mode\n1: The motor is in constant speed mode, which is controlled by parameter 0x2e, and the highest bit 15 is the direction bit\n2: PWM open-loop speed regulation mode, with parameter 0x2c running time parameter control, bit11 as direction bit\n3: In step servo mode, the number of step progress is represented by parameter 0x2a, and the highest bit 15 is the direction bit“
34	0x34	Protective torque	1	20	EPROM	read&write	0	254	1.0%	After entering the overload protection, the output torque, if set to 20, means 20% of the maximum torque
35	0x35	Protection time	1	200	EPROM	read&write	0	254	10ms	The timing time when the current load output exceeds the overload torque and remains. If 200 is set to 2 seconds, the maximum can be set to 2.5 seconds
36	0x36	Overload torque	1	80	EPROM	read&write	0	254	1.0%	The maximum torque threshold of starting overload protection time meter, if set to 80, means 80% of the maximum torque
37	0x37	Speed closed loop P proportional coefficient	1	10	EPROM	read&write	0	254	no	In the motor constant speed mode (mode 1), the speed loop proportional coefficient
38	0x38	Over current protection time	1	200	EPROM	read&write	0	254	10ms	The maximum setting is 254 * 10ms = 2540ms
39	0x39	Velocity closed loop I integral coefficient	1	10	EPROM	read&write	0	254	no	In the motor constant speed mode (mode 1), the speed loop integral coefficient
40	0x28	Torque switch	1	0	SRAM	read&write	0	2	no	Write 0: turn off torque output; write 1: turn on torque output; write 128: current position correction is 2048
41	0x41	acceleration	1	0	SRAM	read&write	0	254	100step/s^2	If it is set to 10, the speed will be changed by 1000 steps per second
42	0x42	Target location	2	0	SRAM	read&write	-32766	32766	step	Each step is a minimum resolution angle, absolute position control mode, the maximum corresponding to the maximum effective angle
44	0x44	Running time	2	0	SRAM	read&write	0	1000	0.10%	
46	0x46	running speed	2	0	SRAM	read&write	0	254	step/s	Number of steps in unit time (per second), 50 steps / second = 0.732 RPM (cycles per minute)
48	0x48	Torque limit	2	1000	SRAM	read&write	0	1000	1.0%	The initial value of power on is assigned by the maximum torque (0x10), which can be modified by the user to control the output of the maximum torque
55	0x55	Lock mark	1	0	SRAM	read&write	0	1	no	Write 0 closes the write lock, and the value written to EPROM address is saved after power failure.\nWrite 1 opens the write lock, and the value written to EPROM address is not saved after power failure
56	0x56	current location	2	0	SRAM	read-only	-1	-1	step	In the absolute position control mode, the maximum value corresponds to the maximum effective angle
58	0x58	Current speed	2	0	SRAM	read-only	-1	-1	step/s	Feedback the current speed of motor rotation, the number of steps in unit time (per second)
60	0x60	Current load	2	0	SRAM	read-only	-1	-1	0.1%	Voltage duty cycle of current control output drive motor
62	0x62	Current voltage	1	0	SRAM	read-only	-1	-1	0.1V	Current servo working voltage
63	0x63	Current temperature	1	0	SRAM	read-only	-1	-1	°C	Current internal operating temperature of the servo
64	0x64	Asynchronous write flag	1	0	SRAM	read-only	-1	-1	no	When using asynchronous write instruction, flag bit
65	0x65	Servo status	1	0	SRAM	read-only	-1	-1	no	Bit0 Bit1 bit2 bit3 bit4 bit5 corresponding bit is set to 1, indicating that the corresponding error occurs,Voltage sensor temperature current angle overload corresponding bit 0 is no phase error.
66	0x66	Mobile sign	1	0	SRAM	read-only	-1	-1	no	When the servo is moving, it is marked as 1, and when the servo is stopped, it is 0
69	0x69	Current current	2	0	SRAM	read-only	-1	-1	6.5mA	The maximum measurable current is 500 * 6.5ma = 3250ma
"#;

#[derive(Debug)]
struct RegisterDescription {
    memory_address: &'static str,
    function: &'static str,
    bytes: &'static str,
    initial_value: &'static str,
    storage_area: &'static str,
    authority: &'static str,
    minimum_value: &'static str,
    maximum_value: &'static str,
    unit: &'static str,
    analysis_of_values: &'static str,
}

const PREFIX: &str = r#"/** Auto-generated code. Do not modify. */

trait Register {
    type Value;
    
    const MEMORY_ADDRESS: u8;
}

"#;
const SUFFIX: &str = r#""#;

#[cfg(test)]
mod codegen {
    use super::*;

    fn dedent(s: &str) -> String {
        let first_indent = s.lines().skip(1).next().unwrap_or("").chars().take_while(|c| c.is_whitespace()).collect::<String>();

        s.replace(&(String::from("\n") + &first_indent), "\n").trim_end_matches(' ').to_string().replace(" \n", "\n")
    }
    fn dedent_last(s: &str) -> String {
        let last_indent = s.lines().last().unwrap_or("").chars().take_while(|c| c.is_whitespace()).collect::<String>();

        s.replace(&(String::from("\n") + &last_indent), "\n").trim_end_matches(' ').to_string().replace(" \n", "\n")
    }

    fn format_code() -> String {
        let mut result = String::new();
        result.push_str(PREFIX);
        let registers: Vec<_> = MEMORY_TABLE_TSV.lines().skip(3).map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let register = RegisterDescription {
                // ignore field 1: it's just the hex representation of the memory address, but they make a bunch of mistakes so it's utterly garbage
                memory_address: fields[0],
                function: fields[2],
                bytes: fields[3],
                initial_value: fields[4],
                storage_area: fields[5],
                authority: fields[6],
                minimum_value: fields[7],
                maximum_value: fields[8],
                unit: fields[9],
                analysis_of_values: fields[10],
            };
            register
        }).collect();

        for register in &registers {
            let struct_name = register.function.split(' ').map(|word| { word[0..1].to_uppercase() + &word[1..] }).collect::<String>();
            let value_type = match register.bytes {
                "1" => "u8",
                "2" => "u16",
                _ => panic!("Unsupported byte count: {}", register.bytes),
            };
            let memory_address = register.memory_address;
            let function = register.function;
            let analysis_of_values = register.analysis_of_values;
            let initial_value = register.initial_value;
            let storage_area = register.storage_area;
            let authority = register.authority;
            let minimum_value = register.minimum_value;
            let maximum_value = register.maximum_value;
            let unit = register.unit;
            result.push_str(&dedent(&format!(r#"
                /**
                 * {function}
                 * 
                 * {analysis_of_values}
                 * 
                 * initial_value: {initial_value}
                 * storage_area: {storage_area}
                 * authority: {authority}
                 * minimum_value: {minimum_value}
                 * maximum_value: {maximum_value}
                 * unit: {unit}
                 */
                pub struct {struct_name};
                impl Register for {struct_name} {{
                    type Value = {value_type};
                    const MEMORY_ADDRESS: u8 = {memory_address};
                }}
            "#)).replace("\n *\n *\n *", "\n *"));
        }

        result.push_str("pub enum RegisterAddress {\n");
        for register in &registers {
            let struct_name = register.function.split(' ').map(|word| { word[0..1].to_uppercase() + &word[1..] }).collect::<String>();
            result.push_str(&format!("    {struct_name}({struct_name}),\n", struct_name = struct_name));
        }
        result.push_str("}\n");
        
        result.push_str(&dedent(r#"
            impl RegisterAddress {"#));

        // from_memory_address()
        result.push_str(&dedent(r#"
            
                pub fn from_memory_address(memory_address: u8) -> Option<Self> {
                    match memory_address {
            "#)[1..]);
        for register in &registers {
            let memory_address = register.memory_address;
            let struct_name = register.function.split(' ').map(|word| { word[0..1].to_uppercase() + &word[1..] }).collect::<String>();
            result.push_str(&format!("            {memory_address} => Some(Self::{struct_name}({struct_name})),\n"));
        }
        result.push_str(&dedent_last(r#"
                    _ => None,
                }
            }
        "#).trim_start_matches("\n"));

        // length()
        result.push_str(&dedent(r#"
            
                pub fn length(&self) -> u8 {
                    match self {
            "#)[1..]);
        for register in &registers {
            let bytes = register.bytes;
            let struct_name = register.function.split(' ').map(|word| { word[0..1].to_uppercase() + &word[1..] }).collect::<String>();
            result.push_str(&format!("            Self::{struct_name}(_) => {bytes},\n"));
        }
        result.push_str(&dedent_last(r#"
                }
            }
        "#).trim_start_matches("\n"));

        // } for impl RegisterAddress
        result.push_str(&dedent(r#"}"#));
        

        result.push_str(SUFFIX);

        result
    }
    /**
     * This is inspired by https://llogiq.github.io/2024/03/28/easy.html
     *
     * I much prefer this approach over the procedural macro approach.
     *
     * Note that generated code must be formatted in a way that rustfmt agrees with.
     * 
     * FIXME: move this into another crate or build script so that `cargo test` doesn't break
     * itself irrecoverably if you output invalid code?
     */
    #[test]
    fn generate_registers() {
        let new_source = format_code();

        let registers_module_path = std::path::PathBuf::from("src/registers.rs");
        if let Ok(old_source) = std::fs::read_to_string(&registers_module_path) {
            if new_source == old_source {
                // everything is up to date
                return;
            }
        }

        // write the new source to the file and fail the test to make sure CI can ensure that
        // the generated code is up to date
        std::fs::write(registers_module_path, new_source).unwrap();
        panic!("Updated generated code. Please commit.");
    }
}
