#include "IrisCompiler.h"
#include "IrisInterpreter.h"
#include "IrisFatalErrorHandler.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include <string>
#include <iostream>

#include <Windows.h>

using namespace std;

typedef struct IR_ExtentionInitializeStruct_tag {
	void* m_pInterpreter;
	void* m_pfThreadGetThreadInfo;
	void* m_pfThreadIsMainThead;
} IR_ExtentionInitializeStruct, *PIR_ExtentionInitializeStruct;

bool ExitFunction() {
	return false;
}

void ShowMessage(const string& strMessage) {
	cout << strMessage << endl;
}

int main(int argc, char** argv) {

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	IrisFatalErrorHandler::CurrentFatalHandler()->SetFatalErrorMessageFuncton(ShowMessage);

	// Compile
	pCompiler->LoadScript("script/test.ir");
	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		remove("script/test.irc");
		return 0;
	}

	// Run
	IrisGC::CurrentGC()->SetGCFlag(false);

	pInterpreter->Initialize();

	pInterpreter->SetExitConditionFunction(ExitFunction);
	pInterpreter->SetCompiler(pCompiler);

	IrisGC::CurrentGC()->ResetNextThreshold();
	IrisGC::CurrentGC()->SetGCFlag(true);

	auto time1 = ::GetTickCount64();

	pInterpreter->Run();

	auto time2 = ::GetTickCount64();

	pInterpreter->ShutDown();

	cout << "\n" << "Iris has finished the script in " << time2 - time1 << "ms. <(£þ¡¦£þ)/¡£" << endl;

	return 0;
}