project(
  'signal-you-messenger',
  'rust',
  version: '1.0.0',
  license: 'GPL-3.0-or-later',
  meson_version: '>= 1.0.0',
  default_options: [
    'warning_level=2',
    'werror=false',
  ],
)

i18n = import('i18n')
gnome = import('gnome')

# Application ID and paths
application_id = 'com.signalyou.Messenger'
gettext_package = meson.project_name()
localedir = get_option('prefix') / get_option('localedir')
datadir = get_option('prefix') / get_option('datadir')
bindir = get_option('prefix') / get_option('bindir')

# Profile-specific settings
profile = get_option('profile')
if profile == 'development'
  application_id = 'com.signalyou.Messenger.Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
else
  version_suffix = ''
endif

# Dependencies
dependency('gtk4', version: '>= 4.12')
dependency('libadwaita-1', version: '>= 1.4')

# Find cargo
cargo = find_program('cargo')

# Configuration data for source files
conf = configuration_data()
conf.set_quoted('APP_ID', application_id)
conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('VERSION', meson.project_version() + version_suffix)
conf.set_quoted('DATADIR', datadir)
conf.set_quoted('PKGDATADIR', datadir / meson.project_name())

# Generate config.rs
configure_file(
  input: 'src/config.rs.in',
  output: 'config.rs',
  configuration: conf,
)

# Subdirectories
subdir('data')
subdir('src')
subdir('po')

# Post-install scripts
gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)

# Summary
summary({
  'Profile': profile,
  'Application ID': application_id,
  'Prefix': get_option('prefix'),
}, section: 'Build')
