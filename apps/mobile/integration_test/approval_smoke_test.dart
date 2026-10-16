import 'package:integration_test/integration_test.dart';
import '../test/approval_test.dart' as approvals;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  approvals.main(deviceRun: true);
}
