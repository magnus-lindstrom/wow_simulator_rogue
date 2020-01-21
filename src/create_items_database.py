import sqlite3

from sqlite3 import Error


def create_connection():
    """ create a database connection to a SQLite database """
    try:
        connection = sqlite3.connect("../db/items.db")
        return connection
    except Error as e:
        print(e)


def create_table(con):
    try:
        cursor_object = con.cursor()
        cursor_object.execute(
            "CREATE TABLE IF NOT EXISTS items ("
            "entry integer unsigned NOT NULL DEFAULT '0' PRIMARY KEY,"
            "class integer unsigned NOT NULL DEFAULT '0',"
            "subclass integer unsigned NOT NULL DEFAULT '0',"
            "name varchar(255) NOT NULL DEFAULT '',"
            "displayid integer unsigned NOT NULL DEFAULT '0',"
            "Quality integer unsigned NOT NULL DEFAULT '0',"
            "Flags integer unsigned NOT NULL DEFAULT '0',"
            "BuyCount integer unsigned NOT NULL DEFAULT '1',"
            "BuyPrice integer unsigned NOT NULL DEFAULT '0',"
            "SellPrice integer unsigned NOT NULL DEFAULT '0',"
            "InventoryType integer unsigned NOT NULL DEFAULT '0',"
            "AllowableClass integer NOT NULL DEFAULT '-1',"
            "AllowableRace integer NOT NULL DEFAULT '-1',"
            "ItemLevel integer unsigned NOT NULL DEFAULT '0',"
            "RequiredLevel integer unsigned NOT NULL DEFAULT '0',"
            "RequiredSkill integer unsigned NOT NULL DEFAULT '0',"
            "RequiredSkillRank integer unsigned NOT NULL DEFAULT '0',"
            "requiredspell integer unsigned NOT NULL DEFAULT '0',"
            "requiredhonorrank integer unsigned NOT NULL DEFAULT '0',"
            "RequiredCityRank integer unsigned NOT NULL DEFAULT '0',"
            "RequiredReputationFaction integer unsigned NOT NULL DEFAULT '0',"
            "RequiredReputationRank integer unsigned NOT NULL DEFAULT '0',"
            "maxcount integer unsigned NOT NULL DEFAULT '0',"
            "stackable integer unsigned NOT NULL DEFAULT '1',"
            "ContainerSlots integer unsigned NOT NULL DEFAULT '0',"
            "stat_type1 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value1 integer NOT NULL DEFAULT '0',"
            "stat_type2 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value2 integer NOT NULL DEFAULT '0',"
            "stat_type3 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value3 integer NOT NULL DEFAULT '0',"
            "stat_type4 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value4 integer NOT NULL DEFAULT '0',"
            "stat_type5 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value5 integer NOT NULL DEFAULT '0',"
            "stat_type6 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value6 integer NOT NULL DEFAULT '0',"
            "stat_type7 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value7 integer NOT NULL DEFAULT '0',"
            "stat_type8 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value8 integer NOT NULL DEFAULT '0',"
            "stat_type9 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value9 integer NOT NULL DEFAULT '0',"
            "stat_type10 integer unsigned NOT NULL DEFAULT '0',"
            "stat_value10 integer NOT NULL DEFAULT '0',"
            "dmg_min1 real NOT NULL DEFAULT '0',"
            "dmg_max1 real NOT NULL DEFAULT '0',"
            "dmg_type1 integer unsigned NOT NULL DEFAULT '0',"
            "dmg_min2 real NOT NULL DEFAULT '0',"
            "dmg_max2 real NOT NULL DEFAULT '0',"
            "dmg_type2 integer unsigned NOT NULL DEFAULT '0',"
            "dmg_min3 real NOT NULL DEFAULT '0',"
            "dmg_max3 real NOT NULL DEFAULT '0',"
            "dmg_type3 integer unsigned NOT NULL DEFAULT '0',"
            "dmg_min4 real NOT NULL DEFAULT '0',"
            "dmg_max4 real NOT NULL DEFAULT '0',"
            "dmg_type4 integer unsigned NOT NULL DEFAULT '0',"
            "dmg_min5 real NOT NULL DEFAULT '0',"
            "dmg_max5 real NOT NULL DEFAULT '0',"
            "dmg_type5 integer unsigned NOT NULL DEFAULT '0',"
            "armor integer unsigned NOT NULL DEFAULT '0',"
            "holy_res integer unsigned NOT NULL DEFAULT '0',"
            "fire_res integer unsigned NOT NULL DEFAULT '0',"
            "nature_res integer unsigned NOT NULL DEFAULT '0',"
            "frost_res integer unsigned NOT NULL DEFAULT '0',"
            "shadow_res integer unsigned NOT NULL DEFAULT '0',"
            "arcane_res integer unsigned NOT NULL DEFAULT '0',"
            "delay integer unsigned NOT NULL DEFAULT '1000',"
            "ammo_type integer unsigned NOT NULL DEFAULT '0',"
            "RangedModRange real NOT NULL DEFAULT '0',"
            "spellid_1 integer unsigned NOT NULL DEFAULT '0',"
            "spelltrigger_1 integer unsigned NOT NULL DEFAULT '0',"
            "spellcharges_1 integer NOT NULL DEFAULT '0',"
            "spellppmRate_1 real NOT NULL DEFAULT '0',"
            "spellcooldown_1 integer NOT NULL DEFAULT '-1',"
            "spellcategory_1 integer unsigned NOT NULL DEFAULT '0',"
            "spellcategorycooldown_1 integer NOT NULL DEFAULT '-1',"
            "spellid_2 integer unsigned NOT NULL DEFAULT '0',"
            "spelltrigger_2 integer unsigned NOT NULL DEFAULT '0',"
            "spellcharges_2 integer NOT NULL DEFAULT '0',"
            "spellppmRate_2 real NOT NULL DEFAULT '0',"
            "spellcooldown_2 integer NOT NULL DEFAULT '-1',"
            "spellcategory_2 integer unsigned NOT NULL DEFAULT '0',"
            "spellcategorycooldown_2 integer NOT NULL DEFAULT '-1',"
            "spellid_3 integer unsigned NOT NULL DEFAULT '0',"
            "spelltrigger_3 integer unsigned NOT NULL DEFAULT '0',"
            "spellcharges_3 integer NOT NULL DEFAULT '0',"
            "spellppmRate_3 real NOT NULL DEFAULT '0',"
            "spellcooldown_3 integer NOT NULL DEFAULT '-1',"
            "spellcategory_3 integer unsigned NOT NULL DEFAULT '0',"
            "spellcategorycooldown_3 integer NOT NULL DEFAULT '-1',"
            "spellid_4 integer unsigned NOT NULL DEFAULT '0',"
            "spelltrigger_4 integer unsigned NOT NULL DEFAULT '0',"
            "spellcharges_4 integer NOT NULL DEFAULT '0',"
            "spellppmRate_4 real NOT NULL DEFAULT '0',"
            "spellcooldown_4 integer NOT NULL DEFAULT '-1',"
            "spellcategory_4 integer unsigned NOT NULL DEFAULT '0',"
            "spellcategorycooldown_4 integer NOT NULL DEFAULT '-1',"
            "spellid_5 integer unsigned NOT NULL DEFAULT '0',"
            "spelltrigger_5 integer unsigned NOT NULL DEFAULT '0',"
            "spellcharges_5 integer NOT NULL DEFAULT '0',"
            "spellppmRate_5 real NOT NULL DEFAULT '0',"
            "spellcooldown_5 integer NOT NULL DEFAULT '-1',"
            "spellcategory_5 integer unsigned NOT NULL DEFAULT '0',"
            "spellcategorycooldown_5 integer NOT NULL DEFAULT '-1',"
            "bonding integer unsigned NOT NULL DEFAULT '0',"
            "description varchar(255) NOT NULL DEFAULT '',"
            "PageText integer unsigned NOT NULL DEFAULT '0',"
            "LanguageID integer unsigned NOT NULL DEFAULT '0',"
            "PageMaterial integer unsigned NOT NULL DEFAULT '0',"
            "startquest integer unsigned NOT NULL DEFAULT '0',"
            "lockid integer unsigned NOT NULL DEFAULT '0',"
            "Material integer NOT NULL DEFAULT '0',"
            "sheath integer unsigned NOT NULL DEFAULT '0',"
            "RandomProperty integer unsigned NOT NULL DEFAULT '0',"
            "block integer unsigned NOT NULL DEFAULT '0',"
            "itemset integer unsigned NOT NULL DEFAULT '0',"
            "MaxDurability integer unsigned NOT NULL DEFAULT '0',"
            "area integer unsigned NOT NULL DEFAULT '0',"
            "Map integer NOT NULL DEFAULT '0',"
            "BagFamily integer NOT NULL DEFAULT '0',"
            "ScriptName varchar(64) NOT NULL DEFAULT '',"
            "DisenchantID integer unsigned NOT NULL DEFAULT '0',"
            "FoodType integer unsigned NOT NULL DEFAULT '0',"
            "minMoneyLoot integer unsigned NOT NULL DEFAULT '0',"
            "maxMoneyLoot integer unsigned NOT NULL DEFAULT '0',"
            "Duration integer unsigned NOT NULL DEFAULT '0',"
            "ExtraFlags integer unsigned NOT NULL DEFAULT '0')"
        )
    except Error as e:
        print(e)


def insert_item(con, item):
    sql = (" INSERT INTO items "
           "(entry,class, subclass, name, displayid, Quality, Flags, BuyCount,"
           "BuyPrice, SellPrice, InventoryType, AllowableClass, AllowableRace, ItemLevel, RequiredLevel,"
           "RequiredSkill, RequiredSkillRank, requiredspell, requiredhonorrank, RequiredCityRank,"
           "RequiredReputationFaction, RequiredReputationRank, maxcount, stackable, ContainerSlots,"
           "stat_type1, stat_value1, stat_type2, stat_value2, stat_type3, stat_value3, stat_type4,"
           "stat_value4, stat_type5, stat_value5, stat_type6, stat_value6, stat_type7, stat_value7,"
           "stat_type8, stat_value8, stat_type9, stat_value9, stat_type10, stat_value10, dmg_min1,"
           "dmg_max1, dmg_type1, dmg_min2, dmg_max2, dmg_type2, dmg_min3, dmg_max3, dmg_type3, dmg_min4,"
           "dmg_max4, dmg_type4, dmg_min5, dmg_max5, dmg_type5, armor, holy_res, fire_res, nature_res,"
           "frost_res, shadow_res, arcane_res, delay, ammo_type, RangedModRange, spellid_1, spelltrigger_1,"
           "spellcharges_1, spellppmRate_1, spellcooldown_1, spellcategory_1, spellcategorycooldown_1, spellid_2,"
           "spelltrigger_2, spellcharges_2, spellppmRate_2, spellcooldown_2, spellcategory_2,"
           "spellcategorycooldown_2, spellid_3, spelltrigger_3, spellcharges_3, spellppmRate_3,"
           "spellcooldown_3, spellcategory_3, spellcategorycooldown_3, spellid_4, spelltrigger_4,"
           "spellcharges_4, spellppmRate_4, spellcooldown_4, spellcategory_4, spellcategorycooldown_4,"
           "spellid_5, spelltrigger_5, spellcharges_5, spellppmRate_5, spellcooldown_5, spellcategory_5,"
           "spellcategorycooldown_5, bonding, description, PageText, LanguageID, PageMaterial, startquest,"
           "lockid, Material, sheath, RandomProperty, block, itemset, MaxDurability, area, Map, BagFamily,"
           "ScriptName, DisenchantID, FoodType, minMoneyLoot, maxMoneyLoot, Duration, ExtraFlags)"
           "VALUES"
           "(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,"
           "?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,"
           "?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
    cur = con.cursor()
    cur.execute(sql, item)
    conn.commit()
    return cur.lastrowid


def dump_db(conn):
    cur = conn.cursor()
    cur.execute("SELECT * FROM items")
    rows = cur.fetchall()
    for row in rows:
        print(row)


def delete_table(conn):
    cur = conn.cursor()
    with conn:
        cur.execute("DROP TABLE items")


if __name__ == '__main__':

    conn = create_connection()
    delete_table(conn)
    create_table(conn)
    dump_db(conn)

    with open('items_to_insert', 'r') as f:
        for cnt, line in enumerate(f):
            res = tuple(line.split(','))
            if len(res) != 128:
                print(line)
            else:
                insert_item(conn, res)
            # if cnt == 20:
                # break

    dump_db(conn)
    # delete_table(conn)












