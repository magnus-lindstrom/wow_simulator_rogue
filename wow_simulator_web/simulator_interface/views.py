from django.template.defaulttags import register
from django.contrib import messages
from django.http import HttpResponseRedirect
from django.shortcuts import render, redirect
from django.urls import reverse
from copy import deepcopy
from distutils.util import strtobool
from django.views.generic import TemplateView
import os
from .forms import MyForm
import yaml


class HomeView(TemplateView):
    template_name = 'home.html'
    REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
    ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')
    WEAPONS_FILE = os.path.join(REPO_ROOT, 'db', 'weapons.yaml')
    TALENTS_FILE = os.path.join(REPO_ROOT, 'db', 'talents.yaml')
    BUFFS_FILE = os.path.join(REPO_ROOT, 'db', 'buffs.yaml')

    CONFIG_FILE_FOLDER = os.path.join(REPO_ROOT, 'configs')

    WEAPON_SLOTS = ['MH', 'OH']

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
        if request.POST.get("generate_config"):
            return self.create_config_file(request)
        elif request.POST.get("load_config"):
            file_name = request.POST.get("file_to_load")
            file_path = os.path.join(self.CONFIG_FILE_FOLDER, file_name)
            content = self._parse_config_file(file_path)
            messages.add_message(request, messages.SUCCESS, f"Config file content: {content}")
            return HttpResponseRedirect('/')

    def create_config_file(self, request):
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
        config_file_path = os.path.join(self.CONFIG_FILE_FOLDER, config_file_name)

        # output dictionaries
        try:
            mh_name = \
            [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'MH'][0]
        except IndexError:
            mh_name = None

        try:
            oh_name = \
            [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'OH'][0]
        except IndexError:
            oh_name = None

        item_dict = {
            'items': {
                'armor_names': [armor_value for armor_value in armor_values if armor_value],
                'mh_name': mh_name,
                'oh_name': oh_name,
            }
        }
        enchant_dict = {
            'enchants': {
                'armor_enchant_names': [armor_enchant_value for armor_enchant_value in armor_enchant_values if
                                        armor_enchant_value],
                'mh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in
                                     weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'MH'],
                'oh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in
                                     weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'OH']
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

        return HttpResponseRedirect('/')

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

    @staticmethod
    def _parse_config_file(config_file):
        with open(config_file, 'r') as config_file:
            content = yaml.load(config_file, yaml.FullLoader)
        return content


class TestView(TemplateView):
    template_name = 'new_home.html'

    REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    CONFIG_FILE_FOLDER = os.path.join(REPO_ROOT, 'configs')

    def get(self, request, *args, **kwargs):

        context = {
            'form': MyForm()
        }

        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        if request.POST.get("generate_config"):
            return self.create_config_file(request)
        elif request.POST.get("load_config"):
            file_name = request.POST.get("file_to_load")
            file_path = os.path.join(self.CONFIG_FILE_FOLDER, file_name)
            content = self._parse_config_file(file_path)

            talents = {f'talents-{name}': value for name, value in content['talents'].items()}
            buffs = {f'buffs-{name}': value for name, value in content['buffs'].items()}

            initial_values = dict()
            initial_values.update(talents)
            initial_values.update(buffs)

            form = MyForm(initial=initial_values)

            messages.add_message(request, messages.SUCCESS, f"{content}")
            return render(request, self.template_name, {'form': form})

    def create_config_file(self, request):
        # input values
        form = MyForm(request.POST)
        if form.is_valid():
            talents = {name.split('-')[1]: int(value) for name, value in form.cleaned_data.items() if name.startswith('talents-')}
            talent_dict = {
                'talents': talents
            }

            buffs = {name.split('-')[1]: bool(strtobool(value)) for name, value in form.cleaned_data.items() if name.startswith('buffs-')}
            buff_dict = {
                'buffs': buffs
            }


        armor_values = request.POST.getlist('armor')
        armor_enchant_values = request.POST.getlist('armor-enchant')
        weapon_enchant_values = request.POST.getlist('weapon-enchant')
        weapon_values = request.POST.getlist('weapon')

        # output file
        config_file_name = request.POST.get('configFileName')
        config_file_path = os.path.join(self.CONFIG_FILE_FOLDER, config_file_name)

        # output dictionaries
        try:
            mh_name = \
            [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'MH'][0]
        except IndexError:
            mh_name = None

        try:
            oh_name = \
            [weapon_value.split('-')[1] for weapon_value in weapon_values if weapon_value.split('-')[0] == 'OH'][0]
        except IndexError:
            oh_name = None

        item_dict = {
            'items': {
                'armor_names': [armor_value for armor_value in armor_values if armor_value],
                'mh_name': mh_name,
                'oh_name': oh_name,
            }
        }
        enchant_dict = {
            'enchants': {
                'armor_enchant_names': [armor_enchant_value for armor_enchant_value in armor_enchant_values if
                                        armor_enchant_value],
                'mh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in
                                     weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'MH'],
                'oh_enchant_names': [weapon_enchant_value.split('-')[1] for weapon_enchant_value in
                                     weapon_enchant_values if weapon_enchant_value.split('-')[0] == 'OH']
            }
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

        return HttpResponseRedirect('/simulator_interface/test')

    @staticmethod
    def _parse_config_file(config_file):
        with open(config_file, 'r') as config_file:
            content = yaml.load(config_file, yaml.FullLoader)
        return content


@register.filter
def get_item(dictionary, key):
    return dictionary.get(key)


@register.filter
def filteritems(form_obj, filter_string):
    filtered = deepcopy(form_obj)
    for field in form_obj:
        if not field.name.startswith(filter_string):
            del filtered.fields[field.name]
    return filtered

