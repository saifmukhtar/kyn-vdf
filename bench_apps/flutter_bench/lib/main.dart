import 'package:flutter/material.dart';
import 'package:flutter_bench/src/rust/api/simple.dart';
import 'package:flutter_bench/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      home: BenchmarkPage(),
    );
  }
}

class BenchmarkPage extends StatefulWidget {
  const BenchmarkPage({super.key});

  @override
  State<BenchmarkPage> createState() => _BenchmarkPageState();
}

class _BenchmarkPageState extends State<BenchmarkPage> {
  bool _isRunning = false;
  String _output = 'Waiting to run...';

  Future<void> _runBenchmark() async {
    setState(() {
      _isRunning = true;
      _output = 'Running native verification (1000 runs) via FFI...';
    });

    try {
      // Offload to background thread via flutter_rust_bridge async
      final result = await runBenchmark(runs: 1000);
      final avgTime = result.totalTimeMs / 1000;

      setState(() {
        _output = '''
Result: ${result.success ? '✅ VALID' : '❌ INVALID'}
Total time: ${(result.totalTimeMs / 1000).toStringAsFixed(2)} seconds
Average time per verification: ${avgTime.toStringAsFixed(2)} ms
(Measured over 1000 runs natively)
''';
      });
    } catch (e) {
      setState(() {
        _output = 'Error: $e';
      });
    } finally {
      setState(() {
        _isRunning = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('kyn-vdf Flutter Benchmark')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                onPressed: _isRunning ? null : _runBenchmark,
                child: const Text('Run Benchmark (1000 runs)'),
              ),
              const SizedBox(height: 20),
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.black87,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  _output,
                  style: const TextStyle(color: Colors.greenAccent, fontSize: 16),
                  textAlign: TextAlign.center,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
