# luaproc

A simple c-like pre-processor for Lua.

![Screenshot_20240824-020858_Termux](https://github.com/user-attachments/assets/e231fd04-9751-4963-a5c0-849201afff7c)

> ![Screenshot_20240824-020925_Termux](https://github.com/user-attachments/assets/2f6537f9-f668-4249-9656-3fa40798b4ce)




## Build

```
cargo build --release
```

## Usage

```
./luaproc (com|run) <path> [-o <path>] [--flags=*,]
```

## Flags

Flags are a way to specify empty macros from the command line, whenever you run or compile a .luap file and specify a list of flags, they are going to be interpreted as:

```
#define <flag1> #end
...
#define <flagN> #end
...<code>
```

These may be useful for debugging flags, testing, and other use cases.

## Available Directives

### `#define`

- **Constant Macros:**
![Screenshot_20240824-021616_Termux](https://github.com/user-attachments/assets/871c09a5-62c0-4232-bbb5-7c70a95549c8)

- **Parameterized Macros:**
![Screenshot_20240824-021647_Termux](https://github.com/user-attachments/assets/8e348e5e-3bf9-4aa4-8c32-1f174c60cc8f)

- **Empty Macros (syntax sugar):**
![Screenshot_20240824-021806_Termux](https://github.com/user-attachments/assets/4b732b42-431a-4bf3-8767-6299aef4b24e)

### `#undef`

![Screenshot_20240824-022033_Termux](https://github.com/user-attachments/assets/fe41ccf9-8a35-411f-a64a-de5892966801)

### `#ifdef`, `#ifndef` and `#else`

![Screenshot_20240824-022250_Termux](https://github.com/user-attachments/assets/ca5edc40-48b3-4c7b-9e27-b1f61d4168f5)

### `#include`

![Screenshot_20240824-022542_Termux](https://github.com/user-attachments/assets/5d4900b0-e58e-4f41-b157-be569de46671)
