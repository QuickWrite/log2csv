# log2csv
A simple log to CSV parsing system

> [!IMPORTANT]
> This project is still in it's infancy. It currently does not work.
>
> And many things can and will change.

## Usage
The program uses a `<name>.l2c` file to configure the way the log parser should work. <br />
At each execution the `.l2c` file and the `<name>.log` file are provided and the resulting `output.csv` will be generated.

This is being done to enable as much configuration possibilities with this small log parser.

## Project TODO:
- [x] Command line interface
- [ ] L2C Parser
  - [x] Key-Value pair system
  - [x] Comments
  - [x] Main RegEx
  - [ ] Output Structure Settings
  - [ ] Advanced settings
    - [ ] Constraints
    - [ ] Sorting
    - [ ] Counting
    - [ ] Grouping
- [ ] Log Parser
  - [ ] RegEx applier
  - [ ] Mapping
  - [ ] Advanced Systems
- [ ] File output
  - [ ] CSV Exporter

## License
This project is licensed under the [GNU GPL License](LICENSE).
