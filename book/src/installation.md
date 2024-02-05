# Installation
Installation is done from source currently. We provide a helper script that will
install Forgery using cargo by cloning the repo and compiling locally. It might
take a couple of minutes to complete.

## Requirements
You will need the following tools installed:
- bash
- curl
- git
- Rust + Cargo

## Installation
### Helper
Use the helper script:
```console
curl https://github.com/Tudmotu/forgery-rs/blob/main/getforgery.sh | bash
```

### Manually
Clone the git repo:
```console
git clone https://github.com/Tudmotu/forgery-rs.git
```

Inside the repo, install using Cargo:
```console
cargo install --locked --path .
```

Optionally, add the Cargo binary directory to your `$PATH` if it's not already
included:
```console
echo 'export PATH=$PATH:~/.cargo/bin' >> ~/.zshrc
```
<div class="warning">
NOTE:<br>
This command might vary depending on your OS and shell. This instruction
assumes a POSIX OS with zsh.
</div>
