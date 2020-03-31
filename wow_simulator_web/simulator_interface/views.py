import os
from copy import deepcopy
from distutils.util import strtobool

import yaml
from django.contrib import messages
from django.shortcuts import render
from django.template.defaulttags import register
from django.views.generic import TemplateView

from .forms import MyForm


class HomeView(TemplateView):
    template_name = 'home.html'

    REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    CONFIG_FILE_FOLDER = os.path.join(REPO_ROOT, 'configs')

    def get(self, request, *args, **kwargs):

        context = {
            'form': MyForm()
        }
        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        if request.POST.get("generate_config"):
            config_file_name = request.POST.get('configFileName')
            config_file_path = os.path.join(self.CONFIG_FILE_FOLDER, config_file_name)

            # preserve submitted data
            form = MyForm(request.POST)
            context = {
                'form': form
            }

            try:
                self.generate_config_file(request, config_file_path)
            except Exception as e:
                messages.add_message(request, messages.ERROR, f"Error while creating file: {e}")
            else:
                messages.add_message(request, messages.SUCCESS, f"Config file created at: {config_file_path}")

            return render(request, self.template_name, context=context)

        elif request.POST.get("load_config"):
            file_name = request.POST.get("file_to_load")
            file_path = os.path.join(self.CONFIG_FILE_FOLDER, file_name)

            initial_values = self._parse_config_file(file_path)
            form = MyForm(initial=initial_values)

            messages.add_message(request, messages.SUCCESS, f"{initial_values}")
            return render(request, self.template_name, {'form': form})

    def generate_config_file(self, request, config_file_path):
        form = MyForm(data=request.POST)

        if form.is_valid():
            talent_dict = self._get_talents_from_form(form)
            buff_dict = self._get_buffs_from_form(form)
            oh_name = self._get_oh_weapon_from_form(form)
            mh_name = self._get_mh_weapon_from_form(form)
            mh_enchants = self._get_mh_enchants_from_form(form)
            oh_enchants = self._get_oh_enchants_from_form(form)
            armor_items_and_slots = self._get_armor_slot_and_items_from_form(form)
            armor_enchants_and_slots = self._get_armor_slot_and_enchants_from_form(form)

        else:
            raise Exception(form.errors)

        item_dict = {
            'items': {
                'armor_names': [armor_item[1] for armor_item in armor_items_and_slots],
                'mh_name': mh_name,
                'oh_name': oh_name,
            }
        }
        enchant_dict = {
            'enchants': {
                'armor_enchant_names': [armor_enchant[1] for armor_enchant in armor_enchants_and_slots],
                'mh_enchant_names': mh_enchants,
                'oh_enchant_names': oh_enchants,
            }
        }

        web_dict = {
            'web': {
                'armor_items': {
                    slot: armor_item for slot, armor_item in armor_items_and_slots
                },
                'armor_enchant_names': {
                    slot: armor_enchant for slot, armor_enchant in armor_enchants_and_slots
                }
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
                yaml.dump(web_dict, config_file)

        except Exception as e:
            raise

    @staticmethod
    def _get_talents_from_form(form):
        talents = {name.split('-')[1]: int(value) for name, value in form.cleaned_data.items() if
                   name.startswith('talents-')}
        talent_dict = {
            'talents': talents
        }
        return talent_dict

    @staticmethod
    def _get_buffs_from_form(form):
        buffs = {name.split('-')[1]: bool(strtobool(value)) for name, value in form.cleaned_data.items() if
                 name.startswith('buffs-')}
        buff_dict = {
            'buffs': buffs
        }
        return buff_dict

    @staticmethod
    def _get_oh_weapon_from_form(form):
        try:
            oh_name = form.cleaned_data['weapons-OH'][0]
        except IndexError:
            oh_name = None

        return oh_name

    @staticmethod
    def _get_mh_weapon_from_form(form):
        try:
            mh_name = form.cleaned_data['weapons-MH'][0]
        except IndexError:
            mh_name = None
        return mh_name

    @staticmethod
    def _get_mh_enchants_from_form(form):
        return form.cleaned_data['weaponsenchants-MH']

    @staticmethod
    def _get_oh_enchants_from_form(form):
        return form.cleaned_data['weaponsenchants-OH']

    @staticmethod
    def _parse_config_file(config_file):
        with open(config_file, 'r') as config_file:
            content = yaml.load(config_file, yaml.FullLoader)

            talents = {f'talents-{name}': value for name, value in content['talents'].items()}
            buffs = {f'buffs-{name}': value for name, value in content['buffs'].items()}
            oh_name = {'weapons-OH': [content['items']['oh_name']]}
            mh_name = {'weapons-MH': [content['items']['mh_name']]}
            oh_enchants = {'weaponsenchants-OH': content['enchants']['oh_enchant_names']}
            mh_enchants = {'weaponsenchants-MH': content['enchants']['mh_enchant_names']}
            armor_items = {f'armors-{slot}': value for slot, value in content['web']['armor_items'].items()}
            armor_enchants = {f'armorsenchants-{slot}': value for slot, value in content['web']['armor_enchant_names'].items()}

            initial_values = dict()
            initial_values.update(talents)
            initial_values.update(buffs)
            initial_values.update(oh_name)
            initial_values.update(mh_name)
            initial_values.update(oh_enchants)
            initial_values.update(mh_enchants)
            initial_values.update(armor_items)
            initial_values.update(armor_enchants)
        return initial_values

    @staticmethod
    def _get_armor_slot_and_items_from_form(form):

        armor_items = list()
        for armorslot, items in form.cleaned_data.items():
            slot = armorslot.split('-')[1]
            if armorslot.startswith('armors-'):
                for item in items:
                    armor_items.append((slot, item))

        return armor_items

    @staticmethod
    def _get_armor_slot_and_enchants_from_form(form):
        armor_enchants = list()

        for armorenchantslot, items in form.cleaned_data.items():
            slot = armorenchantslot.split('-')[1]
            if armorenchantslot.startswith('armorsenchants-'):
                for item in items:
                    armor_enchants.append((slot, item))

        return armor_enchants


# custom filters for django templates
@register.filter
def get_item(dictionary, key):
    return dictionary.get(key)


@register.filter
def filteritemsstartswith(form_obj, filter_string):
    filtered = deepcopy(form_obj)
    for field in form_obj:
        if not field.name.startswith(filter_string):
            del filtered.fields[field.name]
    return filtered


@register.filter
def filteritemsendswith(form_obj, filter_string):
    filtered = deepcopy(form_obj)
    for field in form_obj:
        if not field.name.endswith(filter_string):
            del filtered.fields[field.name]
    return filtered

