use ratatui::style::Color;

use crate::{
    app::{App, INVENTORY_SIZE, PLAYER},
    components::{Item, Object, Position, RenderStatus},
    engine::UseResult,
};

/// moves and item from the gamemap into the player inventory based on object id
pub fn pick_item_up(app: &mut App, id: usize) {
    if app.inventory.len() >= INVENTORY_SIZE {
        app.add_to_log(format!("Cannot hold that many items."), Color::default());
    } else {
        let idx = app.gamemap.object_ids.iter().position(|&x| x == id);
        match idx {
            Some(x) => {
                // add the item to the inventory
                let item_id = app.gamemap.object_ids.swap_remove(x);
                app.inventory.push(item_id);

                // hide it on the map
                let item_obj = app.objects.get_mut(&item_id).unwrap();
                item_obj.render_status = RenderStatus::Hide;

                // print message to the log
                let message = format!("Picked up {}.", item_obj.name);
                app.add_to_log(message, Color::default());
            }
            None => {
                panic!("invalid object id passed to pick_item_up()!")
            }
        }
    }
}

/// drops an item from the inventory back onto the ground
pub fn drop_item(app: &mut App, inventory_idx: usize) {
    if inventory_idx > app.inventory.len() {
        app.add_to_log("No item to drop.", Color::default());
        return;
    }

    // reshow the item on the map, and set its position to the player's position
    let player_pos = app.objects.get(&PLAYER).unwrap().pos.clone();
    let item_id = app.inventory[inventory_idx];
    let item_obj = app.objects.get_mut(&item_id).unwrap();

    app.gamemap.object_ids.push(item_id);
    item_obj.pos = player_pos;
    item_obj.render_status = RenderStatus::ShowInFOV;

    app.inventory.remove(inventory_idx);
}

/// returns the item for a given index in the inventory
pub fn get_item_in_inventory(app: &App, inventory_idx: usize) -> &Item {
    let item_id = app.inventory[inventory_idx];
    match &app.objects.get(&item_id).unwrap().item {
        Some(x) => x,
        None => {
            panic!("get_item_in_inventory() called, but object does not have an item component!")
        }
    }
}

/// returns the object for a given index in the inventory
pub fn get_object_in_inventory(app: &App, inventory_idx: usize) -> &Object {
    let item_id = app.inventory[inventory_idx];
    match app.objects.get(&item_id) {
        Some(x) => x,
        None => {
            panic!("get_object_in_inventory() called, but could not find an object with that id!")
        }
    }
}

/// uses an item from the specified index in the inventory
pub fn use_item(app: &mut App, inventory_idx: usize, target: Option<Position>) -> UseResult {
    let item = get_item_in_inventory(app, inventory_idx).clone();
    let use_result = item.on_use(app, target);

    match use_result {
        UseResult::UsedUp => {
            // delete item after being used
            app.inventory.remove(inventory_idx);
        }
        UseResult::Cancelled => {
            // item wasn't used, don't delete it
        }
        UseResult::Equipped => {
            // try to equip item by moving it from the inventory to the equipment slot

            // get the index that this item is supposed to be equipped in
            let obj = get_object_in_inventory(app, inventory_idx);
            let equip = obj.equipment.as_ref().unwrap();
            let equip_idx = equip.slot as usize;

            // check if the slot is empty or not
            if app.equipment[equip_idx].is_some() {
                app.add_to_log(
                    format!("Already have an item equipped on your {}!", equip.slot),
                    Color::default(),
                );
                return UseResult::Cancelled;
            }

            // if equipment slot isn't empty, equip it
            app.equipment[equip_idx] = Some(app.inventory[inventory_idx]);

            // remove equipped item from inventory
            app.inventory.remove(inventory_idx);
        }
    };

    use_result
}
