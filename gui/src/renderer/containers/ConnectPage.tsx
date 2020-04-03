import { push } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { sprintf } from 'sprintf-js';
import AccountExpiry from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import Connect from '../components/Connect';
import withAppContext, { IAppContext } from '../context';
import { IRelayLocationRedux, RelaySettingsRedux } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

function getRelayName(
  relaySettings: RelaySettingsRedux,
  relayLocations: IRelayLocationRedux[],
): string {
  if ('normal' in relaySettings) {
    const location = relaySettings.normal.location;

    if (location === 'any') {
      return 'Automatic';
    } else if ('country' in location) {
      const country = relayLocations.find(({ code }) => code === location.country);
      if (country) {
        return country.name;
      }
    } else if ('city' in location) {
      const [countryCode, cityCode] = location.city;
      const country = relayLocations.find(({ code }) => code === countryCode);
      if (country) {
        const city = country.cities.find(({ code }) => code === cityCode);
        if (city) {
          return city.name;
        }
      }
    } else if ('hostname' in location) {
      const [countryCode, cityCode, hostname] = location.hostname;
      const country = relayLocations.find(({ code }) => code === countryCode);
      if (country) {
        const city = country.cities.find(({ code }) => code === cityCode);
        if (city) {
          return sprintf(
            // TRANSLATORS: The selected location label displayed on the main view, when a user selected a specific host to connect to.
            // TRANSLATORS: Example: Malmö (se-mma-001)
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(city)s - a city name
            // TRANSLATORS: %(hostname)s - a hostname
            messages.pgettext('connect-container', '%(city)s (%(hostname)s)'),
            {
              city: city.name,
              hostname,
            },
          );
        }
      }
    }

    return 'Unknown';
  } else if (relaySettings.customTunnelEndpoint) {
    return 'Custom';
  } else {
    throw new Error('Unsupported relay settings.');
  }
}

const mapStateToProps = (state: IReduxState) => {
  return {
    accountExpiry: state.account.expiry
      ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
      : undefined,
    loginState: state.account.status,
    selectedRelayName: getRelayName(state.settings.relaySettings, state.settings.relayLocations),
    connection: state.connection,
  };
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const history = bindActionCreators({ push }, dispatch);

  return {
    onSettings: () => {
      history.push('/settings');
    },
    onSelectLocation: () => {
      history.push('/select-location');
    },
    onConnect: async () => {
      try {
        await props.app.connectTunnel();
      } catch (error) {
        log.error(`Failed to connect the tunnel: ${error.message}`);
      }
    },
    onDisconnect: async () => {
      try {
        await props.app.disconnectTunnel();
      } catch (error) {
        log.error(`Failed to disconnect the tunnel: ${error.message}`);
      }
    },
    onReconnect: async () => {
      try {
        await props.app.reconnectTunnel();
      } catch (error) {
        log.error(`Failed to reconnect the tunnel: ${error.message}`);
      }
    },
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(Connect));
