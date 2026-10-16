import 'package:integration_test/integration_test.dart';
import '../test/app_test.dart' as journeys;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  journeys.main(deviceRun: true);
}
