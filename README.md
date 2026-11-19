# Neuro Sport (Agility Sound Timer)

Кроссплатформенный звуковой таймер и тренажер скорости реакции для спорта, единоборств и функционального фитнеса. Приложение написано целиком на **Rust** с использованием **Tauri v2** и **Leptos**.

## Основные возможности

* **Гибкие звуковые раунды:** Настройка длительности раунда, времени отдыха, количества раундов, времени подготовки перед стартом и интервалов подачи сигналов (фиксированных или со случайным отклонением).
* **Интерактивный режим реакции (Touch Mode):** Измерение времени отклика спортсмена от момента подачи звукового сигнала до касания кнопки на экране.
* **Набор голосовых и звуковых команд:** Встроенные в бинарник аудиосигналы для отработки движений по направлениям (8 сторон света) и цветовым маркерам (6 цветов), а также служебные сигналы начала и конца раунда.
* **Полная автономность:** Вся база данных (SQLite) и аудиоресурсы хранятся локально внутри приложения — таймер работает в залах без доступа к интернету.
* **Система аккаунтов и безопасность:** Локальная регистрация, авторизация с управлением сессиями (TTL), смена и восстановление пароля через OTP-код по электронной почте (SMTP).
* **Кроссплатформенность и адаптивный UI:** Единая кодовая база для macOS, Android (планшеты и смартфоны) и iOS (iPhone) с учетом безопасных зон экрана (Safe Area).

## Технологический стек

* **Backend & Core:** Rust, Tauri v2, Tokio
* **Frontend:** Leptos 0.8 (CSR / WebAssembly), Trunk
* **Database:** SQLite (`sqlx` с автоматическими миграциями из папки `migrations/`)
* **Audio Engine:** `rodio` (аудиоресурсы вшиты через `include_bytes!`)
* **Security & Mail:** `bcrypt` / `argon2`, `lettre` (SMTP)

## Структура проекта

```text
neuro_sport/
├── src/                # Фронтенд на Leptos (компоненты, контексты, экраны тренировок и авторизации)
├── src-tauri/          # Нативный бэкенд на Rust (команды Tauri, аудиоплеер rodio, работа с SQLite)
├── migrations/         # SQL-миграции для автоматического развертывания таблиц (user, exercise_stats)
├── public/             # Исходные аудиофайлы (colors/, directions/, prepare/)
├── styles.css          # Адаптивные стили интерфейса
├── Trunk.toml          # Конфигурация сборки WebAssembly-фронтенда
└── Cargo.toml          # Зависимости фронтенда и настройка workspace
```

## Быстрый старт (Разработка на Desktop)

### Предварительные требования
* Установленный **Rust** и `cargo`
* Утилита сборки фронтенда **Trunk** (`cargo install trunk`)
* Tauri CLI v2 (`cargo install tauri-cli`)
* Таргет для WebAssembly (`rustup target add wasm32-unknown-unknown`)

### Запуск в режиме разработки
```bash
cargo tauri dev
```

## Сборка под мобильные платформы

### Android (APK)
1. Установите Android Studio, Android SDK (API 26+) и NDK, а также **OpenJDK 21**.
2. Пропишите переменные окружения `JAVA_HOME`, `ANDROID_HOME`, `NDK_HOME` и `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`.
3. Добавьте таргеты Rust и инициализируйте проект:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   cargo tauri android init
   ```
4. Соберите APK-файл:
   ```bash
   cargo tauri android build --apk
   ```

### iOS (IPA)
1. Установите **Xcode**, платформу iOS SDK и **CocoaPods** (`brew install cocoapods`).
2. Добавьте таргет Rust и инициализируйте проект:
   ```bash
   rustup target add aarch64-apple-ios
   cargo tauri ios init
   ```
3. Укажите `Team` (Apple ID) и подключите системные фреймворки `AudioToolbox`, `CoreAudio`, `AVFAudio` в Xcode.
4. Соберите релизный `.ipa` пакет:
   ```bash
   cargo tauri ios build
   ```

## Соавторство и благодарности

* **Gemini (Google)** — помощь в проектировании архитектуры приложения на Rust (Tauri + Leptos), настройке сборочных пайплайнов для Android и iOS, адаптации мобильного UI и отладке.

## Лицензия

Проект распространяется под лицензией MIT (см. файл `LICENSE`).

---
*Примечание: данный файл `README.md` был сгенерирован с помощью **Gemini**.*
