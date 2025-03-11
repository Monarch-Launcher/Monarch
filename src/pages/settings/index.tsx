import Button from '@_ui/button';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import * as React from 'react';
import { useForm } from 'react-hook-form';
import styled from 'styled-components';

const Section = styled.div`
  display: flex;
  flex-direction: column;
  align-items: flex-start;
`;

const SectionTitle = styled.h3`
  color: ${({ theme }) => theme.colors.primary};
`;

const MonarchSwitch = styled(Switch)`
  input:checked + .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
  }

  .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.secondary};
    border-color: ${({ theme }) => theme.colors.secondary};
  }

  .mantine-Switch-label {
    color: ${({ theme }) => theme.colors.white};
  }
`;

const FormContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1rem;
`;

const ButtonContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
`;

type FormValues = {
  settings: Settings;
  username: string;
  password: string;
};

const SettingsPage = () => {
  const { register, handleSubmit } = useForm<FormValues>();
  const { settings, updateSettings, saveCredentials } = useSettings();

  const onSubmit = React.useCallback(
    async (values: FormValues) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { username, password } = values;
      // Do some if checks

      await saveCredentials(username, password);

      await updateSettings(values.settings);
    },
    [updateSettings, saveCredentials],
  );

  const toggleQuickLaunch = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const updatedSettings: Settings = { ...settings, quicklaunch: {enabled: e.currentTarget.checked, close_shortcut: settings.quicklaunch.close_shortcut, open_shortcut: settings.quicklaunch.open_shortcut, size: settings.quicklaunch.size} };
      // Call backend function
      await updateSettings(updatedSettings);
    },
    [settings],
  );

  const toggleSteam = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const updatedSettings: Settings = { ...settings, steam: {manage: e.currentTarget.checked, game_folders: settings.steam.game_folders, username: settings.steam.username} };
      // Call backend function
      await updateSettings(updatedSettings);
    },
    [settings],
  );

  return (
    <Page title="Settings">
      <Section>
        <SectionTitle>Monarch</SectionTitle>
        <MonarchSwitch
          checked={settings.quicklaunch.enabled}
          onChange={toggleQuickLaunch}
          size="md"
          label="Quicklaunch (Windows and MacOS only)"
          labelPosition="left"
        />
      </Section>
      <Section>
        <SectionTitle>Steam</SectionTitle>
        <MonarchSwitch
          checked={settings.steam.manage}
          onChange={toggleSteam}
          size="md"
          label="Allow Monarch to manage Steam games"
          labelPosition="left"
        />

        <form onSubmit={handleSubmit(onSubmit)}>
          <FormContainer>
            <Input
              placeholder="Steam username"
              variant="filled"
              {...register('username')}
            />
            <Input
              placeholder="Steam password"
              variant="filled"
              type="password"
              {...register('password')}
            />
            <ButtonContainer>
              <Button type="submit" variant="primary">
                Save
              </Button>
            </ButtonContainer>
          </FormContainer>
        </form>
      </Section>
    </Page>
  );
};

export default SettingsPage;
