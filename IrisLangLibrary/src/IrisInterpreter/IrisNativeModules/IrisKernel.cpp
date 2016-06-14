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

			//IrisGC::CurrentGC()->SetGCFlag(false);
			auto pInfo = IrisDevUtil::GetCurrentThreadInfo();

			if (!pCompiler->LoadScript(strFileName)) {
				IrisDevUtil::GroanIrregularWithString(string("Error when requiring the script : " + strFileName + "!").c_str());
				return IrisDevUtil::Nil();
			}

			bool bCompileResult = pCompiler->Generate();
			if (!bCompileResult) {
				IrisDevUtil::GroanIrregularWithString(string("Error when requiring the script : " + strFileName + "!").c_str());
				return IrisDevUtil::Nil();
			}

			//IrisGC::CurrentGC()->ResetNextThreshold();
			//IrisGC::CurrentGC()->SetGCFlag(true);

			auto& vcCodes = pCompiler->GetCodes();

			auto nOldFileIndex = pInfo->m_nCurrentFileIndex;
			pInfo->m_nCurrentFileIndex = IrisCompiler::CurrentCompiler()->GetCurrentFileIndex();

			pInterpreter->PushEnvironment();
			pInterpreter->SetEnvironment(nullptr);

			pInterpreter->RunCode(vcCodes, 0, vcCodes.size());

			pInterpreter->PopEnvironment();
			pInfo->m_nCurrentFileIndex = nOldFileIndex;
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

	//IrisGC::CurrentGC()->SetGCFlag(false);

	if (!pCompiler->LoadScriptString(strScript)) {
		IrisDevUtil::GroanIrregularWithString((string("Error when eval the script string of ") + strScript).c_str());
		return pInterpreter->Nil();
	}

	bool bCompileResult = pCompiler->Generate();
	if (!bCompileResult) {
		IrisDevUtil::GroanIrregularWithString((string("Error when eval the script string of ") + strScript).c_str());
		return pInterpreter->Nil();
	}

	//IrisGC::CurrentGC()->ResetNextThreshold();
	//IrisGC::CurrentGC()->SetGCFlag(true);

	auto& vcCodes = pCompiler->GetEvalCodes();

	pInterpreter->PushEnvironment();
	auto& skStack = IrisDevUtil::GetCurrentThreadInfo()->m_skEnvironmentStack;
	pInterpreter->SetEnvironment(skStack[skStack.size() - 2]);

	pInterpreter->RunCode(vcCodes, 0, vcCodes.size());

	pInterpreter->PopEnvironment();

	return IrisInterpreter::CurrentInterpreter()->Nil();
}

IrisValue IrisKernel::SRand(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	if (ivsVariableValues) {
		if (ivsVariableValues->GetSize() == 1) {
			auto& ivParam = ivsVariableValues->GetValue(0);
			if (IrisDevUtil::CheckClassIsInteger(ivParam)) {
				auto nSRand = IrisDevUtil::GetInt(ivParam);
				srand(nSRand);
			}
			else {
				IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be an Integer.");
			}
		}
		else {
			IrisDevUtil::GroanIrregularWithString("Invaild parameter list.");
		}
	}
	else {
		srand((unsigned int)(::GetTickCount()));
	}
	return IrisDevUtil::Nil();
}

IrisValue IrisKernel::Rand(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto& ivFrom = ivsValues->GetValue(0);
	auto& ivTo = ivsValues->GetValue(1);

	if (!IrisDevUtil::CheckClassIsInteger(ivFrom) && !IrisDevUtil::CheckClassIsFloat(ivFrom)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be an Integer or a Float");
		return IrisDevUtil::Nil();
	}

	if (!IrisDevUtil::CheckClassIsInteger(ivTo) && !IrisDevUtil::CheckClassIsFloat(ivTo)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 2 which must be an Integer or a Float");
		return IrisDevUtil::Nil();
	}

	auto fFrom = IrisDevUtil::CheckClassIsInteger(ivFrom) ? IrisDevUtil::GetInt(ivFrom) : IrisDevUtil::GetFloat(ivFrom);
	auto fTo = IrisDevUtil::CheckClassIsInteger(ivTo) ? IrisDevUtil::GetInt(ivTo) : IrisDevUtil::GetFloat(ivTo);

	return IrisDevUtil::CreateFloat(rand() * 1.0f / RAND_MAX * (fTo - fFrom) + fFrom);
}

IrisKernel::IrisKernel()
{
}


IrisKernel::~IrisKernel()
{
}
