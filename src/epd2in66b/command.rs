/// EPD6IN66 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    DriverOutputControl = 0x01,
    GateDrivingVoltageControl = 0x03,
    SourceDrivingVoltageControl = 0x04,
    InitialCodeSettingOTPProgram = 0x08,
    WriteRegisterForInitialCodeSetting = 0x09,
    ReadRegisterForInitialCodeSetting = 0x0A,
    DeepSleepMode = 0x10,
    DataEntryModeSetting = 0x11,
    BoosterSoftStartControl = 0x0C,
    SWReset = 0x12,
    TemperatureSensorControl = 0x18,
    TemperatureSensorControlWriteToTemperatureRegister = 0x1A,
    MasterActivation = 0x20,
    DisplayUpdateControl1 = 0x21,
    DisplayUpdateControl2 = 0x22,
    WriteRAMBlackWhite = 0x24,
    WriteRAMRed = 0x26,
    WriteVCOMRegister = 0x2C,
    OTPRegisterReadForDisplayOption = 0x2D,
    StatusBitRead = 0x2F,
    ProgramWSOTP = 0x30,
    WriteLUTRegister = 0x32,
    OTPProgramMode = 0x39,
    SetRamXAddressStartEnd = 0x44,
    SetRamYAddressStartEnd = 0x45,
    SetRamXAddressCounter = 0x4E,
    SetRamYAddressCounter = 0x4F,
}

impl Command {
    /// Returns the address of the command
    pub(crate) fn address(self) -> u8 {
        self as u8
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum DeepSleepMode {
    // Sleeps and keeps access to RAM and controller
    Normal = 0x00,

    // Sleeps without access to RAM/controller but keeps RAM content
    Mode1 = 0x01,

    // Same as MODE_1 but RAM content is not kept
    Mode2 = 0x11,
}

#[allow(dead_code, clippy::enum_variant_names)]
pub enum DataEntryModeIncr {
    XDecrYDecr = 0x0,
    XIncrYDecr = 0x1,
    XDecrYIncr = 0x2,
    XIncrYIncr = 0x3,
}

#[allow(dead_code)]
pub enum DataEntryModeDir {
    XDir = 0x0,
    YDir = 0x4,
}

#[allow(dead_code)]
pub enum DisplayRamOption {
    Normal = 0b0000,
    Bypass = 0b0100,
    Inverse = 0b1000,
}

#[allow(dead_code)]
pub enum SourceOutputMode {
    S0ToS175 = 0,
    S8ToS167 = 1,
}
