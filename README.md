<h1 align="center">jim</h1>
<p align="center">
    <em>java instance manager</em>
</p>

## what is it?

toy† instance manager for java.

---

†: i use it, its alright, there are probably better solutions.

## why make?

because it was fun :) and i needed a tool to do it and cbf to google

## how use?

```bash
$ ls
graalvm-ee-java8-21.3.3
openjdk-17.0.1+12

$ jim add *
successfully installed .../graalvm-ee-java8-21.3.3 (0.122s)
successfully installed .../openjdk-17.0.1+12 (0.067s)

$ LOG_LEVEL=DEBUG jim set openjdk-17.0.1+12
info: starting jim 0.2.0 at .../
debug: searching for openjdk-17.0.1+12
debug: found .../openjdk-17.0.1+12
info: setting current instance to openjdk-17.0.1+12
debug: symlink exists, removing...
debug: creating symlink...

$ jim get
openjdk-17.0.1+12

$ jim list
graalvm-ee-java8-21.3.3
openjdk-17.0.1+12
```

## how get?

```bash
$ git clone https://github.com/camuward/jim && cargo install --path jim
```
