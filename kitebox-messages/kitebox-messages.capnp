@0xed55d2845579b93c;

struct AxisData {
    # Represents the AxisData struct from the bmi2 library

    x @0: Int16; # X axis data.
    y @1: Int16; # Y axis data.
    z @2: Int16; # Z axis data.
}

struct ImuData {
    # Represents the Data struct from the bmi2 library

    acc @0: AxisData; # Accelerometer data.
    gyr @1: AxisData; # Gyroscope data.
    time @2: UInt32; # Sensor time.
}

struct Time {
    time @0: UInt32; # time since boot (TODO: units?)
}


struct ReportMessage {
    report: union {
        imuData @0: ImuData;
        time @1: Time;
    }
}
