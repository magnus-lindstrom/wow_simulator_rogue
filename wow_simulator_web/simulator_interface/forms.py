import os

import yaml
from django.forms import forms
from django.forms import CharField, TextInput, MultipleChoiceField, CheckboxInput, BooleanField


class MyForm(forms.Form):

    REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
    ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')
    WEAPONS_FILE = os.path.join(REPO_ROOT, 'db', 'weapons.yaml')
    TALENTS_FILE = os.path.join(REPO_ROOT, 'db', 'talents.yaml')
    BUFFS_FILE = os.path.join(REPO_ROOT, 'db', 'buffs.yaml')

    WEAPON_SLOTS = ['MH', 'OH']

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self._create_talent_fields()
        self._create_buff_fields()

        armor_items = self._parse_armor_file()
        armor_enchant_items, weapon_enchant_items = self._parse_enchants_file()
        weapon_items = self._parse_weapon_file()


        context_empty = {
            'armor': armor_items,
            'weapon': weapon_items,
            'armor_enchant': armor_enchant_items,
            'weapon_enchant': weapon_enchant_items,
        }

    def _create_buff_fields(self):
        buffs_list = self._parse_buffs_file()

        for buff in buffs_list:
            buff_id = buff[0]
            buff_display_name = buff[1]

            self.fields.update({
                f'buffs-{buff_id}': CharField(
                    required=False,
                    label=buff_display_name,
                    widget=CheckboxInput(
                        attrs={
                            'class': 'custom-control-input',
                            'label': buff_display_name,
                            'id': buff_id,
                            'value': buff_id
                        }
                    )
                )
            })

    def _create_talent_fields(self):
        talents_list = self._parse_talent_file()
        for talent in talents_list:
            talent_id = talent[0]
            talent_display_name = talent[1]
            talent_max_value = talent[2]

            self.fields.update({
                f'talents-{talent_id}': CharField(
                    required=False,
                    label=talent_display_name,
                    widget=TextInput(
                        attrs={
                            'type': 'number',
                            'min': "0",
                            'step': "1",
                            'max': talent_max_value,
                            'value': 0,
                            'id': talent_id,
                            'label': talent_display_name,
                            'class': 'w-25',
                        },

                    )
                )
            })

    def _parse_armor_file(self):
        with open(self.ARMOR_FILE, 'r') as armor_file:
            content = yaml.load(armor_file, yaml.FullLoader)

        slots = {}
        for item in content:
            slot_name = content[item]['slot']
            display_name = content[item]['name']
            item_name = item
            if slot_name in slots:
                slots[slot_name].append((item_name, display_name))
            else:
                slots[slot_name] = [(item_name, display_name)]
        return slots

    def _parse_buffs_file(self):
        with open(self.BUFFS_FILE, 'r') as buffs_file:
            content = yaml.load(buffs_file, yaml.FullLoader)

        buffs = []
        for item in content:
            display_name = content[item]['name']
            item_name = item
            buffs.append((item_name, display_name))
        return buffs

    def _parse_enchants_file(self):
        with open(self.ENCHANTS_FILE, 'r') as enchants_file:
            content = yaml.load(enchants_file, yaml.FullLoader)

        def _create_slot_dictionary(slots):
            if slot_name in slots:
                if enchant_type in slots[slot_name]:
                    slots[slot_name][enchant_type].append((item_name, display_name))
                else:
                    slots[slot_name][enchant_type] = [(item_name, display_name)]
            else:
                slots[slot_name] = dict()
                slots[slot_name][enchant_type] = [(item_name, display_name)]
            return slots

        armor_slots = {}
        weapon_slots = {}
        for item in content:
            slot_list = content[item]['slot']
            display_name = content[item]['name']
            item_name = item
            enchant_type = content[item]['enchant_type']

            for slot_name in slot_list:
                if slot_name in self.WEAPON_SLOTS:
                    weapon_slots = _create_slot_dictionary(weapon_slots)
                else:
                    armor_slots = _create_slot_dictionary(armor_slots)

        return armor_slots, weapon_slots

    def _parse_weapon_file(self):
        with open(self.WEAPONS_FILE, 'r') as weapon_file:
            content = yaml.load(weapon_file, yaml.FullLoader)

        slots = {}
        for item in content:
            slot_list = content[item]['slot']
            display_name = content[item]['name']
            item_name = item
            for slot_name in slot_list:
                if slot_name in slots:
                    slots[slot_name].append((item_name, display_name))
                else:
                    slots[slot_name] = [(item_name, display_name)]
        return slots

    def _parse_talent_file(self):
        with open(self.TALENTS_FILE, 'r') as talent_file:
            content = yaml.load(talent_file, yaml.FullLoader)

        talents = []
        for item in content:
            max_value = content[item]['max_points']
            display_name = content[item]['name']
            item_name = item

            talents.append((item_name, display_name, max_value))

        return talents
