@0xed55d2845579b93c;

struct AxisData {
    # Represents the AxisData struct from the bmi2 library

    x @0: Int16; # X axis data.
    y @1: Int16; # Y axis data.
    z @2: Int16; # Z axis data.
}

struct Data {
    # Represents the Data struct from the bmi2 library

    acc @0: AxisData; # Accelerometer data.
    gyr @1: AxisData; # Gyroscope data.
    time @2: UInt32; # Sensor time.
}
