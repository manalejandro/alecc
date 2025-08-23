# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Agregado
- Soporte para más optimizaciones
- Mejores mensajes de error
- Soporte para más extensiones de C

### Cambiado
- Mejorado rendimiento del parser
- Optimizada generación de código

### Corregido
- Problemas de compilación en sistemas de 32 bits
- Manejo de errores en el enlazador

## [0.1.1] - 2025-08-23

### Agregado
- ✅ **Operadores compuestos**: Implementados +=, -=, *=, /= con soporte completo
- ✅ **Operadores bitwise**: Agregados &, |, ^, ~, <<, >> con precedencia correcta
- ✅ **Suite de tests**: Tests de integración completamente funcionales (10/10 pasando)
- ✅ **Benchmarks**: Sistema de benchmarks con métricas de rendimiento
- ✅ **Soporte biblioteca**: Configuración dual binario/biblioteca para tests
- ✅ **Recursión mejorada**: Soporte completo para funciones recursivas (factorial, fibonacci)

### Cambiado
- 🔧 **Limpieza de código**: Eliminados todos los warnings de compilación (32 → 0)
- 📝 **Documentación actualizada**: README con limitaciones actuales clarificadas
- 🏗️ **Arquitectura mejorada**: Mejor separación de módulos con lib.rs

### Corregido
- 🐛 **Headers locales**: Eliminada dependencia del directorio include/ local
- 🔨 **Patrones inalcanzables**: Corregidos match patterns redundantes
- ⚠️ **Imports sin usar**: Limpiados todos los imports y variables no utilizados

### Removido
- ❌ **Directorio include/**: Eliminado directorio include/ y referencias obsoletas

## [0.1.0] - 2025-08-21

### Agregado
- Implementación inicial del compilador ALECC
- Soporte para arquitecturas i386, AMD64 y ARM64
- Lexer completo para C/C++
- Parser básico para programas simples
- Generador de código para las tres arquitecturas
- Sistema de optimización con múltiples niveles (-O0 a -O3, -Os, -Oz)
- Enlazador con soporte para bibliotecas estáticas y dinámicas
- Compatibilidad básica con opciones de GCC
- Interfaz de línea de comandos completa
- Sistema de preprocesado básico
- Soporte para inclusión de archivos de cabecera
- Manejo de errores robusto
- Tests de integración y benchmarks
- Documentación completa en README.md
- Scripts de construcción automatizada
- Configuración para CI/CD con GitHub Actions

### Características Principales
- **Alto Rendimiento**: Escrito en Rust para máxima eficiencia
- **Seguridad**: Manejo seguro de memoria y detección temprana de errores
- **Compatibilidad**: Compatible con opciones de línea de comandos de GCC
- **Multiplataforma**: Soporte nativo para múltiples arquitecturas
- **Optimización**: Sistema avanzado de optimización de código

### Limitaciones Conocidas
- Soporte limitado para características avanzadas de C++
- Preprocesador simplificado
- Algunas optimizaciones están en desarrollo
- Compatibilidad parcial con extensiones específicas de GCC

### Documentación
- README.md completo con ejemplos de uso
- Documentación de API en código
- Ejemplos de programas de prueba
- Guía de contribución

### Herramientas de Desarrollo
- Makefile para tareas comunes
- Scripts de construcción automatizada
- Configuración de EditorConfig
- Benchmarks de rendimiento
- Tests de integración completos
