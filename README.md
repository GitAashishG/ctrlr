# ctrlr

Natural language → terminal commands. Type what you want, get the command, run it.

```
$ nli list files in descending order of size

  ❯ find . -maxdepth 1 -type f -printf '%s\t%p\n' | sort -nr

  Run? (Y/n) y

  4096    ./README.md
  1234    ./Cargo.toml
```

## Install

```bash
# Clone & build
git clone https://github.com/GitAashishG/ctrlr.git
cd ctrlr
cargo build --release

# Add to PATH (pick one)
sudo cp target/release/nli /usr/local/bin/
# or
echo 'export PATH="$PATH:/path/to/ctrlr/target/release"' >> ~/.bashrc
```

## Setup

Set your OpenAI API key (or any compatible API):

```bash
export OPENAI_API_KEY="sk-..."
```

Optionally configure model and base URL:

```bash
export CTRLR_MODEL="gpt-4o-mini"          # default: gpt-4o-mini
export OPENAI_BASE_URL="https://api.openai.com/v1"  # default
```

Add these to your `~/.bashrc` or `~/.zshrc` to persist.

## Usage

```bash
nli <what you want to do in plain english>
```

The tool will:
1. Send your query to the LLM with your OS/shell context
2. Display the suggested command
3. Ask you to confirm before running

### Examples

```bash
nli find all python files modified in the last week
nli compress this folder into a tar.gz
nli show disk usage sorted by size
nli kill the process running on port 3000
nli create a git branch called feature/auth
```

## How it works

~150 lines of Rust. Sends one API call with a system prompt like:

> "Give a terminal command for \<query\> on \<OS\> using \<shell\>"

Returns the raw command, asks Y/n, runs it. That's it.

## Binary size

~2MB stripped. Starts in under 5ms.