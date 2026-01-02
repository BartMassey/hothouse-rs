# Cleveland Music Hothouse signal map
Bart Massey 2026

Taken from the [schematic](https://github.com/clevelandmusicco/HothouseExamples/wiki/Frequently-Asked-Questions#q-where-can-i-find-the-hothouse-schematics)

| Hothouse  | Daisy | crate | MCU  | function      |
| ---:      | ---:  | ---:  | ---: | :---          |
| POT1      | 23    | 16    | PA3  | ADC1          |
| POT2      | 24    | 17    | PB1  | ADC2          |
| POT3      | 25    | 18    | PA7  | ADC3          |
| POT4      | 26    | 19    | PA6  | ADC4          |
| POT5      | 27    | 20    | PC1  | ADC5          |
| POT6      | 28    | 21    | PC4  | ADC6          |
|           |       |       |      |               |
| SW1\_DOWN | 11    | 10    | PB5  | (SPI1\_MOSI)  |
| SW1\_UP   | 10    | 9     | PB4  | (SPI1\_MISO)  |
| SW2\_DOWN | 9     | 8     | PG11 | (SPI1\_SCK)   |
| SW2\_UP   | 8     | 7     | PG10 | (SPI1\_NSS)   |
| SW3\_DOWN | 7     | 6     | PC12 | (SDMMC1\_CK)  |
| SW3\_UP   | 6     | 5     | PD2  | (SDMMC1\_CMD) |
|           |       |       |      |               |
| LED\_1    | 29    | 22    | PA5  | (ADC7/DAC2)   |
| LED\_2    | 30    | 23    | PA4  | (ADC8/DAC1)   |
|           |       |       |      |               |
| FSW\_1    | 32    | 25    | PA0  | (ADC10)       |
| FSW\_2    | 33    | 26    | PD11 | (SAI2\_SD\_A) |

| Hothouse      | Daisy  |
|          ---: |   ---: |
| AUDIO\_IN\_L  | 16     |
| AUDIO\_IN\_R  | 17     |
| AUDIO\_OUT\_L | 18     |
| AUDIO\_OUT\_R | 19     |
