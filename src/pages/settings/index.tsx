import Button from '@_ui/button';
import Page from '@_ui/page';
import { useSettings } from '@global/contexts/settingsProvider';
import { Settings } from '@global/types';
import { Input, Switch } from '@mantine/core';
import { invoke } from '@tauri-apps/api';
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
  const [quickLaunchToggled, setQuickLaunchToggled] = React.useState(true);
  const { register, handleSubmit } = useForm<FormValues>();
  // @ts-expect-error
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { settings, updateSettings } = useSettings();

  const onSubmit = React.useCallback(
    async (values: FormValues) => {
      // @ts-expect-error
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { username, password } = values;
      // Do some if checks

      await updateSettings(values.settings);
    },
    [updateSettings],
  );

  const toggleQuickLaunch = React.useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      setQuickLaunchToggled(e.currentTarget.checked);
      // Call backend function
      await invoke('');
    },
    [],
  );

  return (
    <Page title="Settings">
      <Section>
        <SectionTitle>Monarch</SectionTitle>
        <MonarchSwitch
          checked={quickLaunchToggled}
          onChange={toggleQuickLaunch}
          size="md"
          label="Quicklaunch"
          labelPosition="left"
        />
      </Section>
      <Section>
        <SectionTitle>Steam</SectionTitle>
        <MonarchSwitch
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
