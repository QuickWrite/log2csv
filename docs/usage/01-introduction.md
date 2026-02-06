# Introduction
`log2csv` is a lightweight, configurable tool that transforms plain‑text log files into structured CSV tables.
But this can also be used for anything that is separated by lines and has some sort of structure in each line.

## Structure of the program
It has two different main inputs:
1. The logfile that should be parsed and converted to a CSV file.
2. The file that defines how to parse this CSV file.

The second file is necessary as it is not given how the logfile is structured and how to convert this into a CSV file.

```shell
> log2csv <logfile> <definition.l2c>
```

## Structure of the definition
The definition on how to actually parse the logfile has the file extension of `.l2c` (which is variable).

Each`.l2c` file is separated into key-value pairs. These pairs are separated by the first `:` that can be found:
```yaml
key: value

this can be anything: value also here
```
The key is case insensitive. This means that `regex` and `RegEx` are the same key for `log2csv`:
```yaml
Same key: Here
sAmE KeY: Also here
```

Comments are prefixed by `#` and end with a `\n`:
```yaml
# This is a comment
my key: my value

# Anything after the '#' will be ignored
# Also a
# key : value pair
```

----

To actually create a valid `.l2c` file there have to be two different keys present:
| Key     | Value                                                            |
|---------|------------------------------------------------------------------|
| `regex` | The regular expression that should be used to parse a line.      |
| `order` | The different columns that should be used inside of the CSV file |

The `order` uses the captions defined in the `regex`.

This means that in order to parse a file that consists of Key-Value Pairs like `a:b` a file like this can be created:
```yaml
regex: (?<key>\w):(?<value>\w)
order: key value
```
> [!NOTE]
> This works as anything with the syntax of `(?<name> ...)` is a named capturing group. The parentheses define a capturing group and the `?<...>` has the name of this capturing group defined. <br />
> This means that there are two different capturing groups with the names of `key` and `value`. And these two are later on being used in `order` as a space separated list.
>
> If you want to learn about how to use regular expressions [you can find a RegEx tutorial here](https://www.regexone.com/). 

For a file that just contains `a:b` this would result in a file called `output.csv` with the contents of:
```csv
key,value
a,b
```
