# Asylum
Clean up file and folder names for support on NTFS SMB Windows / Mac / Linux

```sh
git clone http://github.com/microuser/Asylum.git
cd Asylum
cargo build
cd target
./Asylum -help
./Asylum ~/Downloads
```


```sh
echo this is a one liner if we have unicode.
find . -type f -exec rename -v 's/[^0-9A-Za-z.\/]/-/g;s/-+/-/g' {} \;
```
