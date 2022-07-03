# BMS Config Migrator

Tired of hand-editing your BMS config with each update?

Have some crazy idea that a computer can surely figure out what changed between
two text files? Me too!

```
bms-config-migrate.exe \
> --old "E:/Falcon BMS 4.35/User/Config/" \
> --new "E:/Falcon BMS 4.36/User/Config/"
...
// Merged user config:
set g_bAfterburnerDetentClick 1
set g_bEnableGtraining 0
set g_fAmbientmin 0.15
set g_fFOVIncrement 40
set g_sCompatibleChanges "all-C_CHANGE_DYNAMIC_SMART_SCALING-C_CHANGE_CHANGE_STORE_DAMAGE"
set g_sNonCompatibleChanges "all-DURANDAL-NC_CHANGE_DEAGGED_LOADOUT"

// These settings were in the old Falcon BMS.cfg but not the new.
// They might be unused in the new version of BMS,
// or maybe you set them! Clean up any you don't want.
set g_bLocalEnvironmentalDate 0
set g_bServerHostAll 1
set g_bVisorUpByDefault 1
set g_nDeagTimer 6
set g_nReagTimer 6

// These settings were changed from the old Falcon BMS.cfg to the new.
// DELETE THEM IF YOU DIDN'T SET THEM YOURSELF!
// (If you didn't, there's probably a reason they changed.)
set g_bPlayIntroMovie 0
set g_bPrettyScreenShot 1
set g_nHotasPinkyShiftMagnitude 128
set g_nMPStartRestricted 1
set g_nPOV1DeviceID 3
set g_nPOV2DeviceID 5
set g_sPriorityFixes "-76"
```

## What does it do?

Given an `--old` and `--new` BMS config folder (`<BMS Install>/User/Config`),

1. Find `Falcon BMS.cfg` and `Falcon BMS User.cfg` (if the latter exists)

2. Merges the old and new user config, preferring the new if there's conflicts.

3. Finds options in the old config but not the new (maybe you set them?)

4. Finds options that changed from the old config to the new.
   (Maybe you set them, or maybe the defaults changed. Look at these carefully!)

5. Prints that all to stdout - pipe it file, review it,
   and make it your new user config!
