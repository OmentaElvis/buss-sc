# Bussin sanity checker
![banner](assets/bonner.png)

## What's this?

This is a minimal server for bussin protocol that listens
on default ports and prints out the textual representation
of the things you sent to it thinking its ok.

## Who is it for?
Anyone re inventing their own new shiny server/client and 
needs to confirm they are insane by taking on this noble 
quest in the first place.

## What to expect.
If everything you send including the binary data is correct according 
to the protocol spec then its all good otherwise you may see 
unspeakable errors such as the mythical `SEGFAULT` in a rust program.


Here is a sample sane output.

```
$ buss-sc -p 42069
[+] Bussin at port 42069
[+] Received a connection from: 127.0.0.1:43372
[#] Request Header
+-----------------------------------+
 Bussin 1.0 READ /
 Settings count: 2
 0>BodyLength: 11
 1>Host: buss.rizz
 Body:
Happinessss
+-----------------------------------+
[+] Received a connection from: 127.0.0.1:43374
[#] Request Header
+-----------------------------------+
 Bussin 1.0 READ /
 Settings count: 2
 0>BodyLength: 5
 1>Host: buss.rizz
 Body:  
wefwe
+-----------------------------------+
```

## How do I use it?
`--help` will help you.

Goodluck.

## Is it dangerous?
You are the danger.
