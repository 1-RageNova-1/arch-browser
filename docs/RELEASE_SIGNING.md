# Release signing

Release tags are built by `.github/workflows/release.yml`. The workflow deliberately fails rather than publishing an unsigned release.

## Create a dedicated signing key

Create a dedicated OpenPGP key for Arch Browser releases on a trusted machine. Keep an offline backup of the private key and do not commit it to the repository.

Export the private key as ASCII armor and add it as the repository Actions secret `APT_GPG_PRIVATE_KEY`. Add its passphrase as `APT_GPG_PASSPHRASE` if the key is protected. The public key is exported into every signed release as `arch-browser-signing-key.gpg`.

The GitHub CLI can set the secrets without putting the private key in source control:

```bash
gpg --armor --export-secret-keys "Arch Browser Release" | gh secret set APT_GPG_PRIVATE_KEY --repo 1-RageNova-1/arch-browser
read -rsp "Passphrase de la clé GPG: " APT_GPG_PASSPHRASE
printf '%s' "$APT_GPG_PASSPHRASE" | gh secret set APT_GPG_PASSPHRASE --repo 1-RageNova-1/arch-browser
unset APT_GPG_PASSPHRASE
```

If the key has no passphrase, add an empty `APT_GPG_PASSPHRASE` secret or update the workflow accordingly.

## Verify a downloaded package

Download the `.deb`, its `.asc` signature and `arch-browser-signing-key.gpg` from the same GitHub Release. Then run:

```bash
gpg --import arch-browser-signing-key.gpg
gpg --verify arch-browser_0.1.2_amd64.deb.asc arch-browser_0.1.2_amd64.deb
sudo apt install ./arch-browser_0.1.2_amd64.deb
```

A successful `gpg --verify` must show a good signature and the expected Arch Browser release-key fingerprint. Do not trust a public key downloaded from the same location unless its fingerprint has also been verified through a separate trusted channel.

This currently signs the release package and its download. It is not yet a hosted APT repository with signed `InRelease` metadata; that requires a separately configured package-repository host.
