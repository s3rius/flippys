#include "core/log.h"
#include "flippys_scene.h"
#include "gui/modules/submenu.h"
#include "gui/scene_manager.h"
#include "gui/view_dispatcher.h"

typedef enum {
    SelectAnime
} MainMenuItems;

void menu_calllback(void* context, uint32_t index) {
    UNUSED(index);
    FlippysApp* app = (FlippysApp*)context;
    furi_assert(app);
    switch(index) {
    case SelectAnime:
        FURI_LOG_I(APP_NAME, "Starting memes");
        break;
    default:
        FURI_LOG_E(APP_NAME, "Unknown menu item selected: %lu", index);
    }
    scene_manager_handle_back_event(app->scene_manager);
}

void flippys_scene_main_menu_on_enter(void* context) {
    FlippysApp* app = (FlippysApp*)context;
    furi_assert(app);
    /* // Initialize the main menu scene */
    FURI_LOG_I(APP_NAME, "STARTING main menu scene");
    submenu_add_item(app->main_menu, "Start memes", SelectAnime, menu_calllback, app);
    view_dispatcher_switch_to_view(
        app->view_dispatcher,
        scene_manager_get_scene_state(app->scene_manager, FlippysSceneMainMenu));
}

void flippys_scene_main_menu_on_exit(void* context) {
    FlippysApp* app = (FlippysApp*)context;
    furi_assert(app);
    submenu_reset(app->main_menu);
    FURI_LOG_I(APP_NAME, "Closing main menu scene");
}

bool flippys_scene_main_menu_on_event(void* context, SceneManagerEvent event) {
    UNUSED(event);
    FlippysApp* app = (FlippysApp*)context;
    furi_assert(app);
    if(event.type == SceneManagerEventTypeBack) {
        view_dispatcher_stop(app->view_dispatcher);
        scene_manager_stop(app->scene_manager);
        return true;
    }
    return false;
}
