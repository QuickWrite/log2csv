# log2csv

A simple log to CSV parsing system

## Usage

The program uses a `<name>.l2c` file to configure the way the log parser should work. <br />
At each execution the `.l2c` file and the `<name>.log` file are provided and the resulting `output.csv` will be generated.

This is being done to enable as much configuration possibilities with this small log parser.

### Example

Let's say we have a list of different People, their ages and the city they currently live in:

```log
Alice 34 Berlin
Bob 22 Paris
Charlie 41 London
Diana 29 Madrid
Evelyn 58 Rome
Frank 30 Berlin
Grace 27 Lisbon
```

as we now want to convert this data into a CSV file, we can now write a `people.l2c` file where the RegEx is the statement that gets applied for every single line and the order are the different captures that should be in the output.

```yaml
regex: (?<name>\w+)\s+(?<age>\d+)\s+(?<city>\w+)

# Provides the columns of the CSV file.
# This has to match the capture names of the RegEx
order: name age city
```

when we execute this with `log2csv people.txt people.l2c` we will get this as `output.csv`:

```csv
name,age,city
Alice,34,Berlin
Bob,22,Paris
Charlie,41,London
Diana,29,Madrid
Evelyn,58,Rome
Frank,30,Berlin
Grace,27,Lisbon
```

---

But now we see that we just wanted the people that are older than 30 in the output. We can do this by modifying the `people.l2c` like this:

```yaml
regex: (?<name>\w+)\s+(?<age>\d+)\s+(?<city>\w+)

order: name age city

# If a row does not match a given constraint,
# it won't be added to the output.
constraint: age.int > 30
# Multiple constaints can be used. They all have to be true.
```

and now we'll get this:

```csv
name,age,city
Alice,34,Berlin
Charlie,41,London
Evelyn,58,Rome
```

## Project TODO:

- [x] Command line interface
- [ ] L2C Parser
  - [x] Key-Value pair system
  - [x] Comments
  - [x] Main RegEx
  - [ ] Output Structure Settings
  - [ ] Advanced settings
    - [x] Constraints
    - [ ] Sorting
    - [ ] Counting
    - [ ] Grouping
- [ ] Log Parser
  - [x] RegEx applier
  - [ ] Mapping
  - [ ] Advanced Systems
- [x] File output
  - [x] CSV Exporter

## License

This project is licensed under the [GNU GPL License](LICENSE).
