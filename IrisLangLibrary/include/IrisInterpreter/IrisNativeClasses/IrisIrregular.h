#ifndef _H_IRISIRREGULAR_
#define _H_IRISIRREGULAR_

#include "IrisDevHeader.h"

class IrisIrregular : public IIrisClass
{
public:
	static IrisValue InitializeFunction(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue SetLineNumber(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue SetFileName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue SetMessage(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetLineNumber(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetFileName(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	static IrisValue GetMessage(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);
	
	static IrisValue ToString(IrisValue& ivObj, IIrisValues* ivsValues, IIrisValues* ivsVariableValues, IIrisContextEnvironment* pContextEnvironment);

public:

	const char* NativeClassNameDefine() const {
		return "Irregular";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	void Mark(void* pNativeObjectPointer) {
	}

	int GetTrustteeSize(void* pNativePointer) {
		//return sizeof(IrisIrregularTag);
		return 0;
	}

	void* NativeAlloc() {
		//return new IrisIrregularTag();
		return nullptr;
	}

	void NativeFree(void* pNativePointer) {
		//delete static_cast<IrisIrregularTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 3, false);

		IrisDevUtil::AddInstanceMethod(this, "to_string", ToString, 0, false);

		IrisDevUtil::AddGetter(this, "@line_number", GetLineNumber);
		IrisDevUtil::AddGetter(this, "@file_name", GetFileName);
		IrisDevUtil::AddGetter(this, "@message", GetMessage);

		IrisDevUtil::AddSetter(this, "@line_number", SetLineNumber);
		IrisDevUtil::AddSetter(this, "@file_name", SetFileName);
		IrisDevUtil::AddSetter(this, "@message", SetMessage);
	}


	IrisIrregular();
	~IrisIrregular();
};

#endif // !_H_IRISIRREGULAR_
