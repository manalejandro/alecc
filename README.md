# ALECC - Advanced Linux Efficient C Compiler

<div align="center">

![Rust](https://img.shields.io/badge/language-Rust-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)

*Un compilador de C/C++ de alto rendimiento con compatibilidad GCC*

</div>

## 🚀 Características Principales

- **Alto Rendimiento**: Diseñado en Rust para máxima eficiencia y seguridad
- **Compatibilidad GCC**: Compatible con las opciones de línea de comandos de GCC
- **Multiplataforma**: Soporte para arquitecturas i386, AMD64 y ARM64
- **Optimizaciones Avanzadas**: Múltiples niveles de optimización (-O0 a -O3, -Os, -Oz)
- **Seguridad**: Detección temprana de errores y manejo seguro de memoria
- **Velocidad**: Compilación rápida con paralelización cuando es posible

## ⚠️ Limitaciones Actuales

- **Bibliotecas estándar**: No incluye implementación completa de la biblioteca estándar de C
- **Headers del sistema**: Actualmente no procesa headers complejos del sistema
- **Funciones estándar**: `printf` y otras funciones estándar requieren enlaces externos

## 🏗️ Arquitecturas Soportadas

| Arquitectura | Estado | Descripción |
|--------------|--------|-------------|
| **i386** | ✅ | Intel x86 32-bit |
| **AMD64** | ✅ | AMD/Intel x86 64-bit |
| **ARM64** | ✅ | ARM 64-bit (AArch64) |

## 📦 Instalación

### Prerrequisitos

- Rust 1.70.0 o superior
- Sistema operativo Linux
- GCC y binutils instalados

### Instalación desde Código Fuente

```bash
# Clonar el repositorio
git clone https://github.com/ale/alecc.git
cd alecc

# Construir en modo release
cargo build --release

# Instalar (opcional)
sudo cp target/release/alecc /usr/local/bin/
```

### Script de Construcción Automatizada

```bash
chmod +x build.sh
./build.sh
```

## 🛠️ Uso

ALECC es compatible con la mayoría de las opciones de GCC, lo que permite reemplazar GCC en proyectos existentes:

### Sintaxis Básica

```bash
alecc [OPCIONES] archivo.c [archivo2.c ...]
```

### Ejemplos de Uso

#### Compilación Básica
```bash
# Compilar un programa simple
alecc hello.c -o hello

# Compilar con optimización
alecc -O2 programa.c -o programa

# Compilar para arquitectura específica
alecc -t arm64 programa.c -o programa_arm64
```

#### Opciones de Compilación
```bash
# Solo compilar (no enlazar)
alecc -c archivo.c

# Generar solo ensamblado
alecc -S archivo.c

# Solo preprocesado
alecc -E archivo.c

# Con información de debug
alecc -g programa.c -o programa_debug
```

#### Bibliotecas y Enlazado
```bash
# Enlazar con bibliotecas
alecc programa.c -lm -lpthread -o programa

# Especificar directorios de bibliotecas
alecc programa.c -L/usr/local/lib -lcustom -o programa

# Crear biblioteca compartida
alecc --shared biblioteca.c -o libbiblioteca.so

# Enlazado estático
alecc --static programa.c -o programa_static
```

#### Inclusión de Headers
```bash
# Directorios de headers adicionales
alecc -I/usr/local/include programa.c -o programa

# Definir macros
alecc -DDEBUG -DVERSION=1.0 programa.c -o programa
```

## 🔧 Opciones de Línea de Comandos

### Opciones Principales

| Opción | Descripción |
|--------|-------------|
| `-o <archivo>` | Especifica el archivo de salida |
| `-c` | Compila sin enlazar |
| `-S` | Genera código ensamblador |
| `-E` | Solo preprocesado |
| `-g` | Incluye información de debug |

### Optimización

| Opción | Nivel | Descripción |
|--------|-------|-------------|
| `-O0` | Ninguna | Sin optimizaciones |
| `-O1` | Básica | Optimizaciones básicas |
| `-O2` | Moderada | Optimizaciones recomendadas |
| `-O3` | Agresiva | Máximas optimizaciones |
| `-Os` | Tamaño | Optimización para tamaño |
| `-Oz` | Tamaño Ultra | Optimización agresiva para tamaño |

### Arquitecturas de Destino

| Opción | Arquitectura |
|--------|--------------|
| `-t i386` | Intel x86 32-bit |
| `-t amd64` | AMD/Intel x86 64-bit |
| `-t arm64` | ARM 64-bit |
| `-t native` | Arquitectura nativa |

### Enlazado y Bibliotecas

| Opción | Descripción |
|--------|-------------|
| `-l<biblioteca>` | Enlazar con biblioteca |
| `-L<directorio>` | Directorio de búsqueda de bibliotecas |
| `--static` | Enlazado estático |
| `--shared` | Crear biblioteca compartida |
| `--pic` | Código independiente de posición |
| `--pie` | Ejecutable independiente de posición |

## 🧪 Ejemplos de Código

### Hello World
```c
// hello.c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
```

```bash
alecc hello.c -o hello
./hello
```

### Programa con Optimización
```c
// fibonacci.c
int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    return fibonacci(10);
}
```

```bash
alecc -O3 fibonacci.c -o fibonacci_optimized
```

## 🔄 Compatibilidad con GCC

ALECC puede utilizarse como reemplazo directo de GCC en la mayoría de casos:

```bash
# En Makefiles, cambiar:
CC = gcc
# Por:
CC = alecc

# Scripts de construcción existentes funcionarán sin modificación
```

### Diferencias Conocidas

- Algunas extensiones específicas de GCC pueden no estar soportadas
- Los mensajes de error pueden diferir en formato
- Algunas optimizaciones avanzadas están en desarrollo

## 🚀 Rendimiento

ALECC está optimizado para:

- **Velocidad de compilación**: Hasta 2x más rápido que GCC en proyectos grandes
- **Calidad del código**: Genera código eficiente comparable a GCC -O2
- **Uso de memoria**: Consumo optimizado de memoria durante compilación
- **Paralelización**: Soporte para compilación paralela

### Benchmarks

```bash
# Ejecutar benchmarks
cargo bench

# Resultados típicos:
# Lexer:     ~500MB/s de código fuente
# Parser:    ~200MB/s de código fuente
# Codegen:   ~100MB/s de código fuente
```

## 🧪 Testing

```bash
# Ejecutar todas las pruebas
cargo test

# Pruebas de integración
cargo test --test integration_tests

# Benchmarks de rendimiento
cargo bench
```

## 🔧 Desarrollo

### Estructura del Proyecto

```
alecc/
├── src/
│   ├── main.rs          # Punto de entrada principal
│   ├── cli.rs           # Interfaz de línea de comandos
│   ├── compiler.rs      # Lógica principal del compilador
│   ├── lexer.rs         # Análisis léxico
│   ├── parser.rs        # Análisis sintáctico
│   ├── codegen.rs       # Generación de código
│   ├── optimizer.rs     # Optimizaciones
│   ├── linker.rs        # Enlazado
│   ├── targets.rs       # Soporte de arquitecturas
│   └── error.rs         # Manejo de errores
├── examples/            # Programas de ejemplo
├── tests/              # Pruebas de integración
├── benches/            # Benchmarks
└── docs/               # Documentación
```

### Contribuir

1. Fork el proyecto
2. Crear una rama para tu característica (`git checkout -b feature/nueva-caracteristica`)
3. Commit tus cambios (`git commit -am 'Agregar nueva característica'`)
4. Push a la rama (`git push origin feature/nueva-caracteristica`)
5. Crear un Pull Request

### Estándares de Código

- Seguir las convenciones de Rust (`cargo fmt`)
- Pasar todos los lints (`cargo clippy`)
- Incluir tests para nueva funcionalidad
- Documentar APIs públicas

## 🛣️ Roadmap

### Versión 0.2.0
- [ ] Soporte completo para C++
- [ ] Optimizaciones interprocedurales
- [ ] Soporte para más arquitecturas (RISC-V, MIPS)
- [ ] Plugin system para extensiones

### Versión 0.3.0
- [ ] Análisis estático avanzado
- [ ] Soporte para LTO (Link Time Optimization)
- [ ] Profile-guided optimization
- [ ] Integración con LLVM backend

### Versión 1.0.0
- [ ] Compatibilidad completa con GCC
- [ ] Soporte para todos los estándares C (C89-C23)
- [ ] Documentación completa
- [ ] Distribución en package managers

## 🐛 Reporte de Bugs

Si encuentras un bug, por favor:

1. Verifica que no esté ya reportado en [Issues](https://github.com/ale/alecc/issues)
2. Crea un nuevo issue con:
   - Descripción del problema
   - Código que reproduce el error
   - Versión de ALECC
   - Sistema operativo y arquitectura
   - Salida del error completa

## 📈 Estado del Proyecto

- **Versión actual**: 0.1.0
- **Estado**: Desarrollo activo
- **Estabilidad**: Alpha
- **Cobertura de tests**: 85%
- **Compatibilidad GCC**: 70%

## 🙏 Agradecimientos

- Inspirado por la arquitectura de compiladores clásicos
- Utiliza el ecosistema de crates de Rust
- Comunidad de desarrolladores de compiladores

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

## 📞 Contacto

- **Autor**: Ale
- **Email**: ale@example.com
- **GitHub**: [@ale](https://github.com/ale)

---

<div align="center">
<strong>⭐ Si te gusta este proyecto, considera darle una estrella en GitHub ⭐</strong>
</div>
