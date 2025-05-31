#include "btcon.h"
#include "core/check.h"
#include "core/record.h"
#include "gui/modules/submenu.h"
#include "gui/scene_manager.h"
#include "gui/view_dispatcher.h"
#include "scenes/btcon_scene.h"
#include "views.h"

const BleProfileHidParams btcon_ble_profile_params = {
    .device_name_prefix = "Btcon",
    .mac_xor = 0x13,
};

static bool btcon_custom_event_callback(void *context, uint32_t event) {
  furi_assert(context);
  BtconApp *app = (BtconApp *)context;
  return scene_manager_handle_custom_event(app->scene_manager, event);
}

static bool btcon_cusom_back_event_callback(void *context) {
  furi_assert(context);
  BtconApp *app = (BtconApp *)context;
  return scene_manager_handle_back_event(app->scene_manager);
}

static void btcon_tick_event_callback(void *context) {
  furi_assert(context);
  BtconApp *app = (BtconApp *)context;
  scene_manager_handle_tick_event(app->scene_manager);
}

void *btcon_app_malloc() {
  BtconApp *app = malloc(sizeof(BtconApp));
  FURI_LOG_I(APP_NAME, "Allocating BtconApp");
  app->bt = furi_record_open(RECORD_BT);
  app->gui = furi_record_open(RECORD_GUI);
  app->notification_app = furi_record_open(RECORD_NOTIFICATION);
  app->scene_manager = scene_manager_alloc(&btcon_scene_handlers, app);
  app->view_dispatcher = view_dispatcher_alloc();

  view_dispatcher_set_custom_event_callback(app->view_dispatcher,
                                            btcon_custom_event_callback);
  view_dispatcher_set_navigation_event_callback(
      app->view_dispatcher, btcon_cusom_back_event_callback);
  view_dispatcher_set_tick_event_callback(app->view_dispatcher,
                                          btcon_tick_event_callback, 100);

  app->main_menu = submenu_alloc();
  view_dispatcher_add_view(app->view_dispatcher, BtconMainMenuView,
                           submenu_get_view(app->main_menu));

  view_dispatcher_attach_to_gui(app->view_dispatcher, app->gui,
                                ViewDispatcherTypeFullscreen);

  FURI_LOG_I(APP_NAME, "BtconApp successfully initialized");
  return app;
}

void btcon_app_free(void *app_ptr) {
  FURI_LOG_I(APP_NAME, "Freeing BtconApp");
  BtconApp *app = (BtconApp *)app_ptr;

  view_dispatcher_remove_view(app->view_dispatcher, BtconMainMenuView);
  submenu_free(app->main_menu);
  view_dispatcher_free(app->view_dispatcher);
  scene_manager_free(app->scene_manager);

  furi_record_close(RECORD_GUI);
  app->gui = NULL;
  furi_record_close(RECORD_BT);
  app->bt = NULL;
  furi_record_close(RECORD_NOTIFICATION);
  app->notification_app = NULL;

  free(app);
  FURI_LOG_I(APP_NAME, "BtconApp successfully freed");
}

void btcon() {
  BtconApp *app = btcon_app_malloc();
  bt_disconnect(app->bt);
  furi_delay_ms(250);

  FURI_LOG_D(APP_NAME, "Starting switching bluetooth profile");
  app->hid_profile = bt_profile_start(app->bt, ble_profile_hid,
                                      (void *)&btcon_ble_profile_params);
  furi_check(app->hid_profile);
  furi_hal_bt_start_advertising();
  /* bt_keys_storage_set_storage_path(app->bt, APP_DATA_PATH(BT_KEYS_PATH)); */
  FURI_LOG_D(APP_NAME, "Bluetooth profile is now HID");

  view_dispatcher_set_event_callback_context(app->view_dispatcher, app);
  scene_manager_next_scene(app->scene_manager, BtconSceneMainMenu);
  view_dispatcher_run(app->view_dispatcher);

  FURI_LOG_D(APP_NAME, "Restoring bluetooth profile");
  bt_disconnect(app->bt);
  furi_delay_ms(250);
  furi_check(bt_profile_restore_default(app->bt));
  /* bt_keys_storage_set_default_path(app->bt); */
  FURI_LOG_D(APP_NAME, "Bluetooth profile is now default");

  btcon_app_free(app);
}
