import Button from '@_ui/button';
import Page from '@_ui/page';
import { Input, Switch } from '@mantine/core';
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

const Settings = () => {
  return (
    <Page title="Settings">
      <Section>
        <SectionTitle>Monarch</SectionTitle>
        <MonarchSwitch size="md" label="Quicklaunch" labelPosition="left" />
      </Section>
      <Section>
        <SectionTitle>Steam</SectionTitle>
        <MonarchSwitch
          size="md"
          label="Allow Monarch to manage Steam games"
          labelPosition="left"
        />
        <FormContainer>
          <Input placeholder="Steam username" variant="filled" />
          <Input placeholder="Steam password" variant="filled" />

          <ButtonContainer>
            <Button type="button" variant="secondary">
              Reset
            </Button>
            <Button type="button" variant="primary">
              Save
            </Button>
          </ButtonContainer>
        </FormContainer>
      </Section>
    </Page>
  );
};

export default Settings;
