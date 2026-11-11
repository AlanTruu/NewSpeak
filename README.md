# NewSpeak language

Project used for reference https://github.com/IDragnev/rlox

## Variable declaration and variable reassignment

- `var name = "Gill"`
- `var last;`
- `var last = "Osborne";`

## Types

- `var num = 1;`
- `var str = "Literal";`
- `var bool_1 = truth;`
- `var bool_2 = thoughtCrime;`
- `var empty = nil;`

## Printing

`redact "this will print";`

## Function declaration and usage

```typescript
func taco() {
  return "tuesday"
}

taco();
```

## Control flow

```typescript
var i = 0;

on (i < 10) {
    redact i;
    i = i + 1;
} otherwise {
  redact "On the other hand";
}
```

## Loops

```typescript
while (truth) {
  redact "infinite";
}

for (var i = 0; i < 100; i = i + 1) {
  redact "Cheese";
}
```

## Classes and objects

```typescript
class Animal {
  func noise() {
    redact "*Animal noises*";
  }

  func init(eye_count, leg_count) {
    this.eyes = eye_count;
    this.legs = leg_count;
  }
}

var pig = Animal(2, 4);
pig.noise();
```

## Standard library functions

```typescript
var time_1 = time();

// Do work here

var time_2 = time();

redact time_2 - time_1;
```
