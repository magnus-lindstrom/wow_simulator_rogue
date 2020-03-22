from django.contrib import messages
from django.http import HttpResponseRedirect
from django.shortcuts import render, redirect
from django.urls import reverse
from django.views.generic import TemplateView
import os
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')

CONFIG_FILE_FOLDER = os.path.join(REPO_ROOT, 'configs')

WEAPON_SLOTS = ['MH', 'OH']


class HomeView(TemplateView):
    template_name = 'home.html'

    def get(self, request, *args, **kwargs):
        armor_items = self._parse_armor_file()
        armor_enchant_items, weapon_enchant_items = self._parse_enchants_file()

        context = {'armor': armor_items,
                   'armor_enchant': armor_enchant_items,
                   'weapon_enchant': weapon_enchant_items}

        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        armor_values = request.POST.getlist('armor')
        armor_enchant_values = request.POST.getlist('armor-enchant')
        weapon_enchant_values = request.POST.getlist('weapon-enchant')
        config_file_name = request.POST.get('configFileName')
        config_file_path = os.path.join(CONFIG_FILE_FOLDER, config_file_name)
        armor_dict = {'items': {'armor_names': [armor_value for armor_value in armor_values if armor_value]}}
        enchant_dict = {
            'enchants': [
                {'armor_enchant_names': [armor_enchant_value for armor_enchant_value in armor_enchant_values if armor_enchant_value]},
                {'mh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'MH']},
                {'oh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'OH']}
            ]
        }


        try:
            with open(config_file_path, 'w') as config_file:
                yaml.dump(armor_dict, config_file)
            with open(config_file_path, 'a') as config_file:
                yaml.dump(enchant_dict, config_file)
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
