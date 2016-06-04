#include "IrisInterpreter/IrisNativeModules/IrisKernel.h"


recursive_mutex IrisKernel::sm_rmEvalMutex;


IrisValue IrisKernel::Print(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	IrisValue ivString;
	if (ivsVariableValues) {
		for (size_t i = 0; i < ivsVariableValues->GetSize(); ++i) {
			auto& elem = (IrisValue&)ivsVariableValues->GetValue(i);
			if (IrisDevUtil::CheckClassIsString(elem)) {
				cout << IrisDevUtil::GetNativePointer<IrisStringTag*>(elem)->GetString();
			}
			else if (IrisDevUtil::CheckClassIsUniqueString(elem)) {
				cout << IrisDevUtil::GetNativePointer<IrisUniqueStringTag*>(elem)->GetString();
			}
			else {
				ivString = IrisDevUtil::CallMethod(elem, nullptr, "to_string");
				cout << IrisDevUtil::GetNativePointer<IrisStringTag*>(ivString)->GetString();
			}
		}
	}
	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisKernel::Require(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {

	if (!IrisThreadManager::CurrentThreadManager()->IsMainThread()) {
		IrisDevUtil::GroanIrregularWithString("require CAN ONLY be used in MAIN Thread!");
		return IrisDevUtil::Nil();
	}

	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();

	string strCurFileName = pCompiler->GetCurrentFileName();

	if (ivsVariableValues) {
		for (size_t i = 0; i < ivsVariableValues->GetSize(); ++i) {
			auto& elem = (IrisValue&)ivsVariableValues->GetValue(i);
			const string& strFileName = IrisDevUtil::GetNativePointer<IrisStringTag*>(elem)->GetString();
			if (pCompiler->HaveFileRequired(strFileName)) {
				continue;
			}

			IrisGC::CurrentGC()->SetGCFlag(false);
			pCompiler->LoadScript(strFileName);

			bool bCompileResult = pCompiler->Generate();
			if (!bCompileResult) {
				return IrisDevUtil::Nil();
			}

			IrisGC::CurrentGC()->ResetNextThreshold();
			IrisGC::CurrentGC()->SetGCFlag(true);

			auto& vcCodes = pCompiler->GetCodes();

			pInterpreter->PushEnvironment();
			pInterpreter->SetEnvironment(nullptr);

			pInterpreter->RunCode(vcCodes, 0, vcCodes.size(), pCompiler->GetCurrentFileIndex());

			pInterpreter->PopEnvironment();

		}
	}

	pCompiler->SetCurrentFile(strCurFileName);

	return pInterpreter->Nil();
}

IrisValue IrisKernel::Import(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	if (!IrisThreadManager::CurrentThreadManager()->IsMainThread()) {
		IrisDevUtil::GroanIrregularWithString("import CAN ONLY be used in MAIN Thread!");
		return IrisDevUtil::Nil();
	}

	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();

	if (ivsVariableValues) {
		for (size_t i = 0; i < ivsVariableValues->GetSize(); ++i) {
			auto& elem = ivsVariableValues->GetValue(i);
			const string& strFileName = IrisDevUtil::GetNativePointer<IrisStringTag*>(elem)->GetString();
			if (!pInterpreter->LoadExtension(strFileName.c_str())) {
				string strErrorMsg("Error when loading extention from " + strFileName);
				IrisDevUtil::GroanIrregularWithString(strErrorMsg.c_str());
				return IrisDevUtil::False();
			}
		}
	}

	return IrisDevUtil::True();
}

IrisValue IrisKernel::Eval(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {

	lock_guard<recursive_mutex> lgLock(sm_rmEvalMutex);

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	pCompiler->SetLineNumber(0);
	const string& strScript = IrisDevUtil::GetNativePointer<IrisStringTag*>((IrisValue&)ivsValues->GetValue(0))->GetString();

	IrisGC::CurrentGC()->SetGCFlag(false);

	pCompiler->LoadScriptString(strScript);

	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		return pInterpreter->Nil();
	}

	IrisGC::CurrentGC()->ResetNextThreshold();
	IrisGC::CurrentGC()->SetGCFlag(true);

	auto& vcCodes = pCompiler->GetEvalCodes();

	pInterpreter->PushEnvironment();
	auto& skStack = IrisDevUtil::GetCurrentThreadInfo()->m_skEnvironmentStack;
	pInterpreter->SetEnvironment(skStack[skStack.size() - 2]);

	pInterpreter->RunCode(vcCodes, 0, vcCodes.size(), IrisDevUtil::GetCurrentThreadInfo()->m_nCurrentFileIndex);

	pInterpreter->PopEnvironment();

	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisKernel::IrisKernel()
{
}


IrisKernel::~IrisKernel()
{
}
