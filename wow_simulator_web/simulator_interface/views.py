from django.contrib import messages
from django.http import HttpResponseRedirect
from django.shortcuts import render, redirect
from django.urls import reverse
from distutils.util import strtobool
from django.views.generic import TemplateView
import os
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')
WEAPONS_FILE = os.path.join(REPO_ROOT, 'db', 'weapons.yaml')
TALENTS_FILE = os.path.join(REPO_ROOT, 'db', 'talents.yaml')
BUFFS_FILE = os.path.join(REPO_ROOT, 'db', 'buffs.yaml')

CONFIG_FILE_FOLDER = os.path.join(REPO_ROOT, 'configs')

WEAPON_SLOTS = ['MH', 'OH']


class HomeView(TemplateView):
    template_name = 'home.html'

    def get(self, request, *args, **kwargs):
        armor_items = self._parse_armor_file()
        armor_enchant_items, weapon_enchant_items = self._parse_enchants_file()
        weapon_items = self._parse_weapon_file()
        talents_list = self._parse_talent_file()
        buffs_list = self._parse_buffs_file()

        context = {
            'armor': armor_items,
            'weapon': weapon_items,
            'armor_enchant': armor_enchant_items,
            'weapon_enchant': weapon_enchant_items,
            'talents': talents_list,
            'buffs': buffs_list
        }

        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        # input values
        armor_values = request.POST.getlist('armor')
        armor_enchant_values = request.POST.getlist('armor-enchant')
        weapon_enchant_values = request.POST.getlist('weapon-enchant')
        weapon_values = request.POST.getlist('weapon')
        talent_values = request.POST.getlist('talents')
        talent_names = request.POST.getlist('talent_names')
        talents = dict(zip(talent_names, [int(value) for value in talent_values]))
        buff_list = request.POST.getlist('buffs')
        buffs = {item.split('-')[0]: bool(strtobool(item.split('-')[1])) for item in buff_list}

        # output file
        config_file_name = request.POST.get('configFileName')
        config_file_path = os.path.join(CONFIG_FILE_FOLDER, config_file_name)

        # output dictionaries
        item_dict = {
            'items': {
                'armor_names': [armor_value for armor_value in armor_values if armor_value],
                'mh_name': [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'MH'],
                'oh_name': [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'OH'],
            }
        }
        enchant_dict = {
            'enchants': {
                'armor_enchant_names': [armor_enchant_value for armor_enchant_value in armor_enchant_values if armor_enchant_value],
                'mh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'MH'],
                'oh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'OH']
            }
        }

        talent_dict = {
            'talents': talents
        }

        buff_dict = {
            'buffs': buffs
        }

        # output file creation
        try:
            with open(config_file_path, 'w') as config_file:
                yaml.dump(item_dict, config_file)
            with open(config_file_path, 'a') as config_file:
                yaml.dump(enchant_dict, config_file)
                yaml.dump(talent_dict, config_file)
                yaml.dump(buff_dict, config_file)

        except Exception as e:
            messages.add_message(request, messages.ERROR, f"Error while creating file: {e}")
        else:
            messages.add_message(request, messages.SUCCESS, f"Config file created at: {config_file_path}")

        return redirect('/')

    @staticmethod
    def _parse_armor_file():
        with open(ARMOR_FILE, 'r') as armor_file:
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

    @staticmethod
    def _parse_buffs_file():
        with open(BUFFS_FILE, 'r') as buffs_file:
            content = yaml.load(buffs_file, yaml.FullLoader)

        buffs = []
        for item in content:
            display_name = content[item]['name']
            item_name = item
            buffs.append((item_name, display_name))
        return buffs

    @staticmethod
    def _parse_enchants_file():
        with open(ENCHANTS_FILE, 'r') as enchants_file:
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
                if slot_name in WEAPON_SLOTS:
                    weapon_slots = _create_slot_dictionary(weapon_slots)
                else:
                    armor_slots = _create_slot_dictionary(armor_slots)

        return armor_slots, weapon_slots

    @staticmethod
    def _parse_weapon_file():
        with open(WEAPONS_FILE, 'r') as weapon_file:
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

    @staticmethod
    def _parse_talent_file():
        with open(TALENTS_FILE, 'r') as talent_file:
            content = yaml.load(talent_file, yaml.FullLoader)

        talents = []
        for item in content:
            max_value = content[item]['max_points']
            display_name = content[item]['name']
            item_name = item

            talents.append((item_name, display_name, max_value))

        return talents

    @staticmethod
    def _parse_config_file(config_file):
        with open(config_file, 'r') as config_file:
            content = yaml.load(config_file, yaml.FullLoader)
        return content

