# dotchaff

Utility to sort your $HOME into piles.

## Usage

Put some config files to `$HOME/.config/dotchaff`, run, get the list of files
which are not yet unambigously sorted into a particular pile back.

## Config files

See `examples/` directory.

## Proposed categories

* `app` — programs
* `config` — files describing software configuration. If removed, programs revert to default configuration.
* `cache` — files which can be removed and will be automatically recreated.
* `data` — files which contain data produced or consumed by programs and useful independent from them. Cannot be recreated if destroyed.
* `log` — historical program execution data, can be removed with no impact on software functioning
* `runtime` — files needed only during program execution (sockets, file locks and similar). Recreated on program start if needed.
* `state` — files created during software execution, useful, but not critical for functioning (shell histories, lists of recently open documents and similar). Differ from `log` in that the programs make use of `state` data. Differ from `data` being program-specific. Differ from `config` being maintained automatically and not as a result of user's choices of configuration.
* `temp` — temporary files, can be removed with no impact on software functioning.

# License

[ISC](LICENSE).
