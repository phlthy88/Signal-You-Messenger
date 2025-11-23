# Source directory meson build file

# Generate gresources
gnome = import('gnome')

blueprints = custom_target('blueprints',
  input: files(
    'ui/window.blp',
    'ui/chat_list.blp',
    'ui/chat_view.blp',
    'ui/compose_bar.blp',
    'ui/message_row.blp',
    'ui/contact_row.blp',
    'ui/link_device_view.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

resources = gnome.compile_resources(
  'signal-you-messenger',
  'signal-you-messenger.gresource.xml',
  gresource_bundle: true,
  dependencies: blueprints,
  source_dir: ['.', meson.current_build_dir()],
  install: true,
  install_dir: get_option('datadir') / meson.project_name(),
)

# Copy config.rs from build directory
config_rs = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf,
)

# Cargo build
cargo_options = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_options += ['--target-dir', meson.project_build_root() / 'cargo-target']

if get_option('profile') == 'default'
  cargo_options += ['--release']
  rust_target = 'release'
else
  rust_target = 'debug'
endif

cargo_env = environment()
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  depends: resources,
  command: [
    'env',
    'MESON_BUILD_ROOT=' + meson.project_build_root(),
    'MESON_SOURCE_ROOT=' + meson.project_source_root(),
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', meson.project_build_root() / 'cargo-target' / rust_target / meson.project_name(),
    '@OUTPUT@',
  ],
)
