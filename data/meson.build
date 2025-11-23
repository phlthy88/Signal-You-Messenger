# Data directory meson build file

# Desktop file
desktop_file = i18n.merge_file(
  input: 'com.signalyou.Messenger.desktop.in',
  output: 'com.signalyou.Messenger.desktop',
  type: 'desktop',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

# Validate desktop file
desktop_file_validate = find_program('desktop-file-validate', required: false)
if desktop_file_validate.found()
  test('validate-desktop-file',
    desktop_file_validate,
    args: [desktop_file],
  )
endif

# Appstream metainfo
metainfo_file = i18n.merge_file(
  input: 'com.signalyou.Messenger.metainfo.xml.in',
  output: 'com.signalyou.Messenger.metainfo.xml',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

# Validate appstream file
appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-appstream',
    appstreamcli,
    args: ['validate', '--no-net', '--explain', metainfo_file],
  )
endif

# GSettings schema
install_data(
  'com.signalyou.Messenger.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

# Icons
subdir('icons')
