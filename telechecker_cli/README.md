
# `telechecker` (CLI)

A rust port of [BellingCat's `telegram-phone-number-checker`](https://github.com/bellingcat/telegram-phone-number-checker/tree/main). 

`telechecker` is a CLI tool used to check if the provided phone numbers are connected to Telegram, retrieving and outputting information about those users. Currently only outputs in JSON format.


## Installation

### GitHub

Currently available for Linux, macos, and Windows on the latest github release, found [here: https://github.com/orph3usLyre/telechecker/releases/latest](https://github.com/orph3usLyre/telechecker/releases/latest), under the name `telechecker_cli`. 

### Build
* Note that this requires having the rust compiler installed, see https://www.rust-lang.org/tools/install for information.


```sh
git clone https://github.com/orph3usLyre/telechecker.git
cargo build -p telechecker_cli --release

# run
./target/release/<YOUR_PLATFORM>/telechecker_cli
```


## Usage/Examples

```bash
Usage: telechecker_cli [OPTIONS] <PHONE_NUMBERS_FROM_FILE|--phone-numbers <PHONE_NUMBERS_ARGS>>

Arguments:
  [PHONE_NUMBERS_FROM_FILE]  Phone numbers to check (provided as file)

Options:
  -u, --user-phone <USER_PHONE_NUMBER>
          User's phone number (associated with a Telegram account)
  -n, --phone-numbers <PHONE_NUMBERS_ARGS>
          Phone numbers to check (provided as arguments)
      --api-id <API_ID>
          User's api id. If not provided, the API_ID must be found inside the `.env` file
      --api-hash <API_HASH>
          User's api hash. If not provided, the API_HASH must be found inside the `.env` file
  -o, --output <OUTPUT_FILE>
          Output file name. Default: `results.json`
      --preserve-contact
          Whether the provided input numbers should be preserved as contacts after info retrieval. Default: false
  -p, --print
          Whether the JSON output should be printed to stdout. Default: false
  -d, --dry-run
          Dry run: Logs the user in but does not retrieve any data (useful with the verbosity flag to see configuration parameters)
  -v...
          Verbosity flag (counted): i.e. `-v`: little output -vvvv: lots of output
  -h, --help
          Print help
  -V, --version
          Print version
```

### Basic usage
```bash
# Command-line provided phone numbers should be comma delimited
telechecker -u +11234567890 -n +9872135342,+0918423843,+132414321
```

```bash
# You may also set the PHONE_NUMBER variable in your .env to avoid providing it with -u
telechecker -n +9872135342,+0918423843,+132414321
```

You can also provide `telechecker` with a file of phone numbers instead of the `-n` flag:
```bash
# Numbers within a file should be new-line delimited
telechecker -u +11234567890 phone_numbers.txt
```

Use the `-v` flag to control level of detail `telechecker` outputs:
```bash
# lots of debug info
telechecker -u +11234567890 -vvv phone_numbers.txt
```

Use the `-o` flag to designate an output file name (default is `results.json`):
```bash
telechecker -u +11234567890 -o my_results.json phone_numbers.txt
```

Use the `--api-id` and `--api-hash` flags to provide the `API_ID` and `API_HASH` as command line arguments rather than inside a `.env` file.
```bash
telechecker -u +11234567890 --api-id YOUR_API_KEY --api-hash YOUR_API_HASH phone_numbers.txt
```

For more information, see
```bash
telechecker --help
```


## Requirements

In order to use `telechecker`, you will need to provide a phone number that has an associated telegram account when prompted. 

The user also needs an `API_ID` and `API_HASH`. These can be retrieved after creating a Telegram developer's account. Follow the steps here: https://core.telegram.org/api/obtaining_api_id#obtaining-api-id. 

These values must be placed in a file named `.env`, inside the folder with the program. 

#### Example `.env`
```env
API_ID='98237402'
API_HASH='100e7ca8f666b89884031a690c4c95bd'
```

The user may also place their phone number inside the `.env` file:
```env
PHONE_NUMBER='+11234567890'
```

or provide it as a command line argument:
```
telechecker --user-phone +11234567890 ...
```

or provide it when prompted, if not supplied.



