use ratatui::style::Color;

use crate::{
    app::{App, INVENTORY_SIZE, PLAYER},
    components::{Item, Object, Position},
    engine::UseResult,
};

/// moves and item from the gamemap into the player inventory based on object id
pub fn pick_item_up(app: &mut App, id: usize) {
    if app.inventory.len() >= INVENTORY_SIZE {
        app.add_to_log(format!("Cannot hold that many items."), Color::default());
    } else {
        // remove it from the map
        let item_pos = app.gamemap.get_position(id).unwrap();
        app.gamemap.remove_item(item_pos.x, item_pos.y);

        // add the item to the inventory
        app.inventory.push(id);

        // print a message to log
        let item_obj = app.objects.get(&id).unwrap();
        let message = format!("Picked up {}.", item_obj.name);
        app.add_to_log(message, Color::default());
    }
}

/// drops an item from the inventory back onto the ground
pub fn drop_item(app: &mut App, inventory_idx: usize) {
    if inventory_idx >= app.inventory.len() {
        app.add_to_log("No item to drop.", Color::default());
        return;
    }

    // reshow the item on the map, and set its position to the player's position
    let pos = app.gamemap.get_position(PLAYER).unwrap();
    let tile = app.gamemap.get_ref(pos.x, pos.y);
    if tile.item.is_some() {
        app.add_to_log("No space to drop item.", Color::default());
        return;
    }

    let item_id = app.inventory[inventory_idx];
    app.gamemap.place_item(item_id, pos.x, pos.y);

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
